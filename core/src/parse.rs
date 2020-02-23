use crate::{
    ast::{
        source::{LabelDecl, Program, Statement},
        Jump, Label, LangValue, Node, Operator, RegisterRef, SpanNode, StackId,
        StackRef, UserRegisterId, ValueSource,
    },
    error::{CompileError, SourceErrorWrapper, WithSource},
    util::Span,
    Compiler,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_while1},
    character::complete::{char, digit1, line_ending, space0, space1},
    combinator::{all_consuming, cut, map, map_res, opt, recognize},
    error::{context, convert_error, ParseError, VerboseError},
    lib::std::ops::RangeTo,
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    AsChar, Compare, IResult, InputTake, InputTakeAtPosition, Offset, Slice,
};
use nom_locate::{position, LocatedSpan};
use std::iter;

type RawSpan<'a> = LocatedSpan<&'a str>;
type ParseResult<'a, T> = IResult<RawSpan<'a>, T, VerboseError<RawSpan<'a>>>;

/// A trait for parsing into AST nodes. Any AST node that can be parsed from the
/// source should implement this trait.
trait Parse<'a>: Sized {
    /// Attempt to parse the input into the AST node. This is generally not
    /// called directly, only from `parse_node`. Generally, this does NOT parse
    /// any surrounding whitespace, just the minimum amount of the input to
    /// complete the node.
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self>;

    /// Attempt to parse the input into the AST node, and include source span
    /// metadata as well.
    fn parse_node(input: RawSpan<'a>) -> ParseResult<'a, SpanNode<Self>> {
        let new_input = input; // need to copy so we can compare old pos vs new
        let (i, value) = Self::parse(new_input)?;

        let index = input.offset(&i);
        let raw_span = input.slice(..index);
        let (i, end_position) = position(i)?;

        let span = Span {
            offset: raw_span.location_offset(),
            length: raw_span.fragment().len(),
            start_line: raw_span.location_line() as usize,
            start_col: raw_span.get_column(),
            end_line: end_position.location_line() as usize,
            end_col: end_position.get_column(),
        };
        Ok((i, Node(value, span)))
    }
}

// covers StackId and UserRegisterId
impl<'a> Parse<'a> for usize {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        map_res(digit1, |s: RawSpan| s.fragment().parse::<usize>())(input)
    }
}

// covers StackId and UserRegisterId
impl<'a> Parse<'a> for LangValue {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        map_res(recognize(tuple((opt(char('-')), digit1))), |s: RawSpan| {
            s.fragment().parse::<LangValue>()
        })(input)
    }
}

impl<'a> Parse<'a> for Label {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        context(
            "Label",
            map(
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
                |s: RawSpan| Label::from(*s.fragment()),
            ),
        )(input)
    }
}

impl<'a> Parse<'a> for LabelDecl {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        map(terminated(Label::parse, tag(":")), LabelDecl)(input)
    }
}

impl<'a> Parse<'a> for StackRef {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        context(
            "Stack",
            map(preceded(tag_no_case("S"), StackId::parse), StackRef),
        )(input)
    }
}

impl<'a> Parse<'a> for RegisterRef {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        context(
            "Register",
            alt((
                // "RLI" => RegisterRef::InputLength
                map(tag_no_case("RLI"), |_| RegisterRef::InputLength),
                // "RSx" => RegisterRef::StackLength(x)
                map(
                    preceded(tag_no_case("RS"), cut(StackId::parse)),
                    RegisterRef::StackLength,
                ),
                // "RXx" => RegisterRef::User(x)
                map(
                    preceded(tag_no_case("RX"), cut(UserRegisterId::parse)),
                    RegisterRef::User,
                ),
            )),
        )(input)
    }
}

impl<'a> Parse<'a> for ValueSource<Span> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        alt((
            // "1" => const value
            map(LangValue::parse_node, ValueSource::Const),
            // "RX1" => register
            map(RegisterRef::parse_node, ValueSource::Register),
        ))(input)
    }
}

impl<'a> Parse<'a> for Operator<Span> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        alt((
            tag_with_args(
                "READ",
                one_arg(RegisterRef::parse_node),
                Operator::Read,
            ),
            tag_with_args(
                "WRITE",
                one_arg(ValueSource::parse_node),
                Operator::Write,
            ),
            tag_with_args(
                "SET",
                two_args(RegisterRef::parse_node, ValueSource::parse_node),
                |(dst, src)| Operator::Set(dst, src),
            ),
            tag_with_args(
                "ADD",
                two_args(RegisterRef::parse_node, ValueSource::parse_node),
                |(dst, src)| Operator::Add(dst, src),
            ),
            tag_with_args(
                "SUB",
                two_args(RegisterRef::parse_node, ValueSource::parse_node),
                |(dst, src)| Operator::Sub(dst, src),
            ),
            tag_with_args(
                "MUL",
                two_args(RegisterRef::parse_node, ValueSource::parse_node),
                |(dst, src)| Operator::Mul(dst, src),
            ),
            tag_with_args(
                "CMP",
                three_args(
                    RegisterRef::parse_node,
                    ValueSource::parse_node,
                    ValueSource::parse_node,
                ),
                |(dst, src_1, src_2)| Operator::Cmp(dst, src_1, src_2),
            ),
            tag_with_args(
                "PUSH",
                two_args(ValueSource::parse_node, StackRef::parse_node),
                |(src, stack)| Operator::Push(src, stack),
            ),
            tag_with_args(
                "POP",
                two_args(StackRef::parse_node, RegisterRef::parse_node),
                |(stack, dst)| Operator::Pop(stack, dst),
            ),
        ))(input)
    }
}

impl<'a> Parse<'a> for Jump<Span> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        alt((
            map(tag_no_case("JMP"), |_| Jump::Jmp),
            tag_with_args("JEZ", one_arg(ValueSource::parse_node), Jump::Jez),
            tag_with_args("JNZ", one_arg(ValueSource::parse_node), Jump::Jnz),
            tag_with_args("JGZ", one_arg(ValueSource::parse_node), Jump::Jgz),
            tag_with_args("JLZ", one_arg(ValueSource::parse_node), Jump::Jlz),
        ))(input)
    }
}

impl<'a> Parse<'a> for Statement<Span> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        context(
            "Statement",
            alt((
                map(LabelDecl::parse_node, Statement::Label),
                map(Operator::parse_node, Statement::Operator),
                // semi-hack, necessary because of how the AST is organized to
                // share code between source and compiled
                map(
                    // TODO make the arg parser more generic for this
                    separated_pair(Jump::parse_node, space1, Label::parse_node),
                    |(jmp, lbl)| Statement::Jump(jmp, lbl),
                ),
            )),
        )(input)
    }
}

impl<'a> Parse<'a> for Program<Span> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        map(
            // separated_list doesn't work properly so we have to do this
            all_consuming(tuple((
                many0(terminated(line, line_ending)),
                opt(line),
            ))),
            // filter out None lines
            |(lines, last_line)| Program {
                body: lines
                    .into_iter()
                    .chain(iter::once(last_line.unwrap_or(None)))
                    .filter_map(std::convert::identity)
                    .collect(),
            },
        )(input)
    }
}

// ===== Combinators =====

fn one_arg<I, O, E: ParseError<I>, F>(
    arg_parser: F,
) -> impl Fn(I) -> IResult<I, O, E>
where
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Fn(I) -> IResult<I, O, E>,
{
    preceded(space1, arg_parser)
}

fn two_args<I, O1, O2, E: ParseError<I>, F, G>(
    arg_parser_one: F,
    arg_parser_two: G,
) -> impl Fn(I) -> IResult<I, (O1, O2), E>
where
    I: InputTakeAtPosition + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Fn(I) -> IResult<I, O1, E>,
    G: Fn(I) -> IResult<I, O2, E>,
{
    tuple((one_arg(arg_parser_one), one_arg(arg_parser_two)))
}

fn three_args<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    arg_parser_one: F,
    arg_parser_two: G,
    arg_parser_three: H,
) -> impl Fn(I) -> IResult<I, (O1, O2, O3), E>
where
    I: InputTakeAtPosition + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
    F: Fn(I) -> IResult<I, O1, E>,
    G: Fn(I) -> IResult<I, O2, E>,
    H: Fn(I) -> IResult<I, O3, E>,
{
    tuple((
        one_arg(arg_parser_one),
        one_arg(arg_parser_two),
        one_arg(arg_parser_three),
    ))
}

/// Parses one instruction (operator or jump) keyword and arguments. Uses the
/// passed parser to parse the arguments, then passes those through the mapper
/// to get a value.
fn tag_with_args<'a, I: 'a, O, Args, ArgParser, Mapper, E>(
    instr_name: &'static str,
    arg_parser: ArgParser,
    mapper: Mapper,
) -> impl Fn(I) -> IResult<I, O, E>
where
    I: InputTake + Clone + Compare<&'static str> + Slice<RangeTo<usize>>,
    E: ParseError<I>,
    ArgParser: Fn(I) -> IResult<I, Args, E>,
    Mapper: Fn(Args) -> O,
{
    map(
        preceded(
            context(instr_name, tag_no_case(instr_name)),
            context(instr_name, cut(arg_parser)),
        ),
        mapper,
    )
}

// ===== Parsers =====

/// Parses a line comment, which starts with a ; and runs to the end of the
/// line. This terminates at the line ending, but does _not_ consume it.
fn line_comment(input: RawSpan) -> ParseResult<'_, ()> {
    map(
        context("Comment", preceded(char(';'), many0(is_not("\r\n")))),
        |_| (), // throw the comment away
    )(input)
}

/// Parse a single line, up to but not including the line ending.
fn line(input: RawSpan) -> ParseResult<'_, Option<SpanNode<Statement<Span>>>> {
    delimited(
        space0,
        opt(Statement::parse_node),
        tuple((space0, opt(line_comment))),
    )(input)
}

fn parse(
    input: &str,
) -> Result<Program<Span>, Vec<SourceErrorWrapper<CompileError>>> {
    match Program::parse(RawSpan::new(input)) {
        Ok((_, program)) => Ok(program),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            let new_error: VerboseError<&str> = VerboseError {
                errors: e
                    .errors
                    .into_iter()
                    .map(|(raw_span, error_kind)| {
                        (*raw_span.fragment(), error_kind)
                    })
                    .collect(),
            };
            // TODO make this less shit
            Err(vec![SourceErrorWrapper::new(
                CompileError::ParseError(convert_error(input, new_error)),
                Span {
                    offset: 0,
                    length: 0,
                    start_line: 1,
                    start_col: 1,
                    end_line: 1,
                    end_col: 1,
                },
                input,
            )])
        }
        // only possible in streaming mode
        Err(nom::Err::Incomplete(_needed)) => unreachable!(),
    }
}

impl Compiler<()> {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub(crate) fn parse(
        self,
    ) -> Result<Compiler<Program<Span>>, WithSource<CompileError>> {
        match parse(&self.source) {
            // Ok(program) => Ok(self.replace_ast(program)),
            Ok(program) => Ok(Compiler {
                source: self.source,
                hardware_spec: self.hardware_spec,
                ast: program,
            }),
            Err(errors) => Err(WithSource::new(errors, self.source)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to make it a bit easier to create spans for tests
    fn span(
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Span {
        Span {
            // The test implementation of PartialEq doesn't check these fields
            offset: 0,
            length: 0,

            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(parse("").unwrap().body, vec![]);
        assert_eq!(parse("\n\n\n").unwrap().body, vec![]);
        assert_eq!(parse("  ").unwrap().body, vec![]);
        assert_eq!(parse("  \n  ").unwrap().body, vec![]);
    }

    #[test]
    fn test_parse_labels() {
        assert_eq!(
            parse(
                "
                LBL:
                LBL1:
                LBL_WITH_UNDERSCORE:
                1LBL:
                "
            )
            .unwrap()
            .body,
            vec![
                Node(
                    Statement::Label(Node(
                        LabelDecl("LBL".into()),
                        span(2, 17, 2, 21)
                    )),
                    span(2, 17, 2, 21)
                ),
                Node(
                    Statement::Label(Node(
                        LabelDecl("LBL1".into()),
                        span(3, 17, 3, 22)
                    )),
                    span(3, 17, 3, 22)
                ),
                Node(
                    Statement::Label(Node(
                        LabelDecl("LBL_WITH_UNDERSCORE".into()),
                        span(4, 17, 4, 37)
                    )),
                    span(4, 17, 4, 37)
                ),
                Node(
                    Statement::Label(Node(
                        LabelDecl("1LBL".into()),
                        span(5, 17, 5, 22)
                    )),
                    span(5, 17, 5, 22)
                ),
            ]
        );
    }

    #[test]
    fn test_parse_read_write() {
        assert_eq!(
            parse(
                "
                ReAd RX0
                WrItE RX0
                "
            )
            .unwrap()
            .body,
            vec![
                Node(
                    Statement::Operator(Node(
                        Operator::Read(Node(
                            RegisterRef::User(0),
                            span(2, 22, 2, 25)
                        ),),
                        span(2, 17, 2, 25)
                    )),
                    span(2, 17, 2, 25)
                ),
                Node(
                    Statement::Operator(Node(
                        Operator::Write(Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(0),
                                span(3, 23, 3, 26)
                            )),
                            span(3, 23, 3, 26)
                        )),
                        span(3, 17, 3, 26)
                    )),
                    span(3, 17, 3, 26)
                )
            ]
        );
    }

    #[test]
    fn test_set_and_registers() {
        assert_eq!(
            parse(
                "
                Set RX1 4
                SET RX1 RLI
                SET RX1 RS0
                "
            )
            .unwrap()
            .body,
            vec![
                Node(
                    Statement::Operator(Node(
                        Operator::Set(
                            Node(RegisterRef::User(1), span(2, 21, 2, 24)),
                            Node(
                                ValueSource::Const(Node(4, span(2, 25, 2, 26))),
                                span(2, 25, 2, 26)
                            )
                        ),
                        span(2, 17, 2, 26)
                    ),),
                    span(2, 17, 2, 26)
                ),
                Node(
                    Statement::Operator(Node(
                        Operator::Set(
                            Node(RegisterRef::User(1), span(3, 21, 3, 24)),
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::InputLength,
                                    span(3, 25, 3, 28)
                                ),),
                                span(3, 25, 3, 28)
                            )
                        ),
                        span(3, 17, 3, 28)
                    ),),
                    span(3, 17, 3, 28)
                ),
                Node(
                    Statement::Operator(Node(
                        Operator::Set(
                            Node(RegisterRef::User(1), span(4, 21, 4, 24)),
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::StackLength(0),
                                    span(4, 25, 4, 28)
                                ),),
                                span(4, 25, 4, 28)
                            )
                        ),
                        span(4, 17, 4, 28)
                    ),),
                    span(4, 17, 4, 28)
                ),
            ]
        );
    }

    #[test]
    fn test_add() {
        assert_eq!(
            parse("Add RX1 RX4").unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Add(
                        Node(RegisterRef::User(1), span(1, 5, 1, 8)),
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(4),
                                span(1, 9, 1, 12)
                            )),
                            span(1, 9, 1, 12)
                        )
                    ),
                    span(1, 1, 1, 12)
                )),
                span(1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_neg_literal() {
        assert_eq!(
            parse("Add RX1 -10").unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Add(
                        Node(RegisterRef::User(1), span(1, 5, 1, 8)),
                        Node(
                            ValueSource::Const(Node(-10, span(1, 9, 1, 12))),
                            span(1, 9, 1, 12)
                        )
                    ),
                    span(1, 1, 1, 12)
                )),
                span(1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_parse_lang_val_max() {
        let source = format!("Add RX1 {}", std::i32::MAX);
        assert_eq!(
            parse(&source).unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Add(
                        Node(RegisterRef::User(1), span(1, 5, 1, 8)),
                        Node(
                            ValueSource::Const(Node(
                                std::i32::MAX,
                                span(1, 9, 1, 19)
                            )),
                            span(1, 9, 1, 19)
                        )
                    ),
                    span(1, 1, 1, 19)
                )),
                span(1, 1, 1, 19)
            )]
        );
    }

    #[test]
    fn test_parse_lang_val_min() {
        let source = format!("Add RX1 {}", std::i32::MIN);
        assert_eq!(
            parse(&source).unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Add(
                        Node(RegisterRef::User(1), span(1, 5, 1, 8)),
                        Node(
                            ValueSource::Const(Node(
                                std::i32::MIN,
                                span(1, 9, 1, 20)
                            )),
                            span(1, 9, 1, 20)
                        )
                    ),
                    span(1, 1, 1, 20)
                )),
                span(1, 1, 1, 20)
            )]
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            parse("Sub RX1 RX4").unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Sub(
                        Node(RegisterRef::User(1), span(1, 5, 1, 8)),
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(4),
                                span(1, 9, 1, 12)
                            )),
                            span(1, 9, 1, 12)
                        )
                    ),
                    span(1, 1, 1, 12)
                )),
                span(1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            parse("Mul RX1 RX4").unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Mul(
                        Node(RegisterRef::User(1), span(1, 5, 1, 8)),
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(4),
                                span(1, 9, 1, 12)
                            )),
                            span(1, 9, 1, 12)
                        )
                    ),
                    span(1, 1, 1, 12)
                )),
                span(1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_cmp() {
        assert_eq!(
            parse("CMP RX0 5 10").unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Cmp(
                        Node(RegisterRef::User(0), span(1, 5, 1, 8)),
                        Node(
                            ValueSource::Const(Node(5, span(1, 9, 1, 10))),
                            span(1, 9, 1, 10)
                        ),
                        Node(
                            ValueSource::Const(Node(10, span(1, 11, 1, 13))),
                            span(1, 11, 1, 13)
                        )
                    ),
                    span(1, 1, 1, 13)
                )),
                span(1, 1, 1, 13)
            )]
        );
    }

    #[test]
    fn test_push() {
        assert_eq!(
            parse("Push RX2 S4").unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Push(
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(2),
                                span(1, 6, 1, 9)
                            )),
                            span(1, 6, 1, 9)
                        ),
                        Node(StackRef(4), span(1, 10, 1, 12))
                    ),
                    span(1, 1, 1, 12)
                )),
                span(1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_pop() {
        assert_eq!(
            parse("Pop S4 RX2").unwrap().body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Pop(
                        Node(StackRef(4), span(1, 5, 1, 7)),
                        Node(RegisterRef::User(2), span(1, 8, 1, 11)),
                    ),
                    span(1, 1, 1, 11)
                )),
                span(1, 1, 1, 11)
            )]
        );
    }

    #[test]
    fn test_jumps() {
        assert_eq!(
            parse(
                "
                JMP LBL
                JEZ RX0 LBL
                JNZ RX0 LBL
                JLZ RX0 LBL
                JGZ RX0 LBL
                "
            )
            .unwrap()
            .body,
            vec![
                Node(
                    Statement::Jump(
                        Node(Jump::Jmp, span(2, 17, 2, 20)),
                        Node("LBL".into(), span(2, 21, 2, 24)),
                    ),
                    span(2, 17, 2, 24)
                ),
                Node(
                    Statement::Jump(
                        Node(
                            Jump::Jez(Node(
                                ValueSource::Register(Node(
                                    RegisterRef::User(0),
                                    span(3, 21, 3, 24)
                                )),
                                span(3, 21, 3, 24)
                            )),
                            span(3, 17, 3, 24)
                        ),
                        Node("LBL".into(), span(3, 25, 3, 28)),
                    ),
                    span(3, 17, 3, 28)
                ),
                Node(
                    Statement::Jump(
                        Node(
                            Jump::Jnz(Node(
                                ValueSource::Register(Node(
                                    RegisterRef::User(0),
                                    span(4, 21, 4, 24)
                                )),
                                span(4, 21, 4, 24)
                            )),
                            span(4, 17, 4, 24)
                        ),
                        Node("LBL".into(), span(4, 25, 4, 28)),
                    ),
                    span(4, 17, 4, 28)
                ),
                Node(
                    Statement::Jump(
                        Node(
                            Jump::Jlz(Node(
                                ValueSource::Register(Node(
                                    RegisterRef::User(0),
                                    span(5, 21, 5, 24)
                                )),
                                span(5, 21, 5, 24)
                            )),
                            span(5, 17, 5, 24)
                        ),
                        Node("LBL".into(), span(5, 25, 5, 28)),
                    ),
                    span(5, 17, 5, 28)
                ),
                Node(
                    Statement::Jump(
                        Node(
                            Jump::Jgz(Node(
                                ValueSource::Register(Node(
                                    RegisterRef::User(0),
                                    span(6, 21, 6, 24)
                                )),
                                span(6, 21, 6, 24)
                            )),
                            span(6, 17, 6, 24)
                        ),
                        Node("LBL".into(), span(6, 25, 6, 28)),
                    ),
                    span(6, 17, 6, 28)
                ),
            ]
        )
    }

    #[test]
    fn test_comments() {
        assert_eq!(
            parse(
                "
                ; comment over here
                Add RX1 RX4 ; comment here
                "
            )
            .unwrap()
            .body,
            vec![Node(
                Statement::Operator(Node(
                    Operator::Add(
                        Node(RegisterRef::User(1), span(3, 21, 3, 24)),
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(4),
                                span(3, 25, 3, 28)
                            )),
                            span(3, 25, 3, 28)
                        )
                    ),
                    span(3, 17, 3, 28)
                )),
                span(3, 17, 3, 28)
            )]
        );
    }

    #[test]
    fn test_parse_simple_file() {
        assert_eq!(
            parse(
                "
                ;comment start
                Read RX0
                ; comment poop
                Set RX0 2 ;comment more poop
                Write RX0
                ; comment pog
                "
            )
            .unwrap()
            .body,
            vec![
                Node(
                    Statement::Operator(Node(
                        Operator::Read(Node(
                            RegisterRef::User(0),
                            span(3, 22, 3, 25)
                        )),
                        span(3, 17, 3, 25)
                    )),
                    span(3, 17, 3, 25)
                ),
                Node(
                    Statement::Operator(Node(
                        Operator::Set(
                            Node(RegisterRef::User(0), span(5, 21, 5, 24)),
                            Node(
                                ValueSource::Const(Node(2, span(5, 25, 5, 26))),
                                span(5, 25, 5, 26)
                            )
                        ),
                        span(5, 17, 5, 26)
                    ),),
                    span(5, 17, 5, 26)
                ),
                Node(
                    Statement::Operator(Node(
                        Operator::Write(Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(0),
                                span(6, 23, 6, 26)
                            )),
                            span(6, 23, 6, 26)
                        )),
                        span(6, 17, 6, 26)
                    )),
                    span(6, 17, 6, 26)
                )
            ]
        );
    }
}
