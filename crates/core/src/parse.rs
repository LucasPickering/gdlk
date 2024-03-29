use crate::{
    ast::{
        source::{LabelDecl, Program, Statement},
        Instruction, Label, LangValue, Node, RegisterRef, SpanNode, StackId,
        StackRef, UserRegisterId, ValueSource,
    },
    consts::{
        INPUT_LENGTH_REGISTER_REF, NULL_REGISTER_REF,
        STACK_LENGTH_REGISTER_REF_TAG, STACK_REF_TAG, USER_REGISTER_REF_TAG,
    },
    error::{CompileError, SourceErrorWrapper, WithSource},
    util::{RawSpan, Span},
    Compiler,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_while1},
    character::complete::{char, digit1, line_ending, space0, space1},
    combinator::{all_consuming, cut, map, map_res, opt, peek, recognize},
    error::{
        context, make_error, ErrorKind, ParseError, VerboseError,
        VerboseErrorKind,
    },
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult, Offset, Slice,
};

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

        Ok((i, Node(value, Span::from_raw_span(&raw_span))))
    }
}

// covers StackId and UserRegisterId
impl<'a> Parse<'a> for usize {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        map_res(digit1, |s: RawSpan| {
            let frag = s.fragment();

            // If the string has unnecessary leading zeroes, reject it. Use an
            // empty error for convenience, its value won't be used anyway.
            if frag.len() > 1 && frag.starts_with('0') {
                Err(())
            } else {
                s.fragment().parse::<usize>().map_err(|_| ())
            }
        })(input)
    }
}

impl<'a> Parse<'a> for LangValue {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        map_res(recognize(tuple((opt(char('-')), digit1))), |s: RawSpan| {
            s.fragment().parse::<LangValue>()
        })(input)
    }
}

impl<'a> Parse<'a> for Label {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        map(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            |s: RawSpan| Label::from(*s.fragment()),
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
        map(
            preceded(tag_no_case(STACK_REF_TAG), StackId::parse),
            StackRef,
        )(input)
    }
}

impl<'a> Parse<'a> for RegisterRef {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        alt((
            // "RZR" => RegisterRef::Null
            map(tag_no_case(NULL_REGISTER_REF), |_| RegisterRef::Null),
            // "RLI" => RegisterRef::InputLength
            map(tag_no_case(INPUT_LENGTH_REGISTER_REF), |_| {
                RegisterRef::InputLength
            }),
            // "RSx" => RegisterRef::StackLength(x)
            map(
                preceded(
                    tag_no_case(STACK_LENGTH_REGISTER_REF_TAG),
                    cut(StackId::parse),
                ),
                RegisterRef::StackLength,
            ),
            // "RXx" => RegisterRef::User(x)
            map(
                preceded(
                    tag_no_case(USER_REGISTER_REF_TAG),
                    cut(UserRegisterId::parse),
                ),
                RegisterRef::User,
            ),
        ))(input)
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

impl<'a> Parse<'a> for Instruction<Span> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        alt((
            tag_with_args("READ", register_ref_arg, Instruction::Read),
            tag_with_args("WRITE", value_source_arg, Instruction::Write),
            tag_with_args(
                "SET",
                tuple((register_ref_arg, value_source_arg)),
                |(dst, src)| Instruction::Set(dst, src),
            ),
            tag_with_args(
                "ADD",
                tuple((register_ref_arg, value_source_arg)),
                |(dst, src)| Instruction::Add(dst, src),
            ),
            tag_with_args(
                "SUB",
                tuple((register_ref_arg, value_source_arg)),
                |(dst, src)| Instruction::Sub(dst, src),
            ),
            tag_with_args(
                "MUL",
                tuple((register_ref_arg, value_source_arg)),
                |(dst, src)| Instruction::Mul(dst, src),
            ),
            tag_with_args(
                "DIV",
                tuple((register_ref_arg, value_source_arg)),
                |(dst, src)| Instruction::Div(dst, src),
            ),
            tag_with_args(
                "CMP",
                tuple((register_ref_arg, value_source_arg, value_source_arg)),
                |(dst, src_1, src_2)| Instruction::Cmp(dst, src_1, src_2),
            ),
            tag_with_args(
                "PUSH",
                tuple((value_source_arg, stack_ref_arg)),
                |(src, stack)| Instruction::Push(src, stack),
            ),
            tag_with_args(
                "POP",
                tuple((stack_ref_arg, register_ref_arg)),
                |(stack, dst)| Instruction::Pop(stack, dst),
            ),
            tag_with_args("JMP", label_arg, Instruction::Jmp),
            tag_with_args(
                "JEZ",
                tuple((value_source_arg, label_arg)),
                |(val_src, label)| Instruction::Jez(val_src, label),
            ),
            tag_with_args(
                "JNZ",
                tuple((value_source_arg, label_arg)),
                |(val_src, label)| Instruction::Jnz(val_src, label),
            ),
            tag_with_args(
                "JGZ",
                tuple((value_source_arg, label_arg)),
                |(val_src, label)| Instruction::Jgz(val_src, label),
            ),
            tag_with_args(
                "JLZ",
                tuple((value_source_arg, label_arg)),
                |(val_src, label)| Instruction::Jlz(val_src, label),
            ),
        ))(input)
    }
}

impl<'a> Parse<'a> for Statement<Span> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        alt((
            map(LabelDecl::parse_node, Statement::Label),
            map(Instruction::parse_node, Statement::Instruction),
        ))(input)
    }
}

impl<'a> Parse<'a> for Program<Span> {
    fn parse(input: RawSpan<'a>) -> ParseResult<'a, Self> {
        context(
            "program",
            map_res(
                all_consuming(many1(line)),
                // filter out None lines
                |lines| {
                    let body: Vec<_> = lines.into_iter().flatten().collect();

                    // If the program is empty, that's no bueno
                    if body.is_empty() {
                        Err(
                            make_error::<RawSpan<'a>, VerboseError<RawSpan<'a>>>(
                                input,
                                // The error we pick here doesn't matter much
                                // since the message will be determined by
                                // the context
                                ErrorKind::Many1,
                            ),
                        )
                    } else {
                        Ok(Program { body })
                    }
                },
            ),
        )(input)
    }
}

// ===== Combinators =====

fn arg<'a, O, F>(
    context_label: &'static str,
    arg_parser: F,
) -> impl FnMut(RawSpan<'a>) -> ParseResult<'a, O>
where
    F: Fn(RawSpan<'a>) -> ParseResult<'a, O>,
{
    // Include the context twice - the outer one when the arg is completely
    // missing, and the inner one will be used when there's an error in the arg
    // itself. Using two lets us get better error spans.
    context(
        context_label,
        delimited(
            space1,
            context(context_label, arg_parser),
            stmt_token_terminator,
        ),
    )
}

/// Parses one instruction keyword and arguments. Uses the passed parser to
/// parse the arguments, then passes those through the mapper to get a value.
/// For single-arg instructions, the arg parser can just be for the argument's
/// type. For multi-arg instructions, you probably want a `tuple()` parser with
/// each arg's parser within.
///
/// TODO make this take a tuple of arg parsers instead
fn tag_with_args<'a, O, Arg, ArgParser, Mapper>(
    instr_name: &'static str,
    arg_parser: ArgParser,
    mapper: Mapper,
) -> impl FnMut(RawSpan<'a>) -> ParseResult<'a, O>
where
    ArgParser: FnMut(RawSpan<'a>) -> ParseResult<'a, Arg>,
    Mapper: FnMut(Arg) -> O,
{
    map(
        preceded(
            // instruction name
            terminated(tag_no_case(instr_name), stmt_token_terminator),
            // arguments
            context(instr_name, cut(arg_parser)),
        ),
        mapper,
    )
}

// ===== Parsers =====

/// Parse a [RegisterRef] argument to an instruction
fn register_ref_arg(input: RawSpan) -> ParseResult<'_, SpanNode<RegisterRef>> {
    arg("register reference", RegisterRef::parse_node)(input)
}

/// Parse a [StackRef] argument to an instruction
fn stack_ref_arg(input: RawSpan) -> ParseResult<'_, SpanNode<StackRef>> {
    arg("stack reference", StackRef::parse_node)(input)
}

/// Parse a [ValueSource] argument to an instruction
fn value_source_arg(
    input: RawSpan,
) -> ParseResult<'_, SpanNode<ValueSource<Span>>> {
    arg("value", ValueSource::parse_node)(input)
}

/// Parse a [Label] argument to an instruction
fn label_arg(input: RawSpan) -> ParseResult<'_, SpanNode<Label>> {
    arg("label", Label::parse_node)(input)
}

/// The terminator that always follows a token in a statement (which is an
/// instruction, argument, or label declaration). This does not consume the
/// terminator, just check that it exists.
fn stmt_token_terminator(input: RawSpan) -> ParseResult<'_, RawSpan> {
    // we don't want to eat a trailing newline, just check if it's there
    peek(alt((space1, eol_or_eof)))(input)
}

/// Parse a line ending, or return success if the input is empty (we've reached
/// the end of the file).
fn eol_or_eof(input: RawSpan) -> ParseResult<'_, RawSpan> {
    if input.fragment().is_empty() {
        Ok((input, input))
    } else {
        line_ending(input)
    }
}

/// Parses a line comment, which starts with a ; and runs to the end of the
/// line. This terminates at the line ending, but does _not_ consume it.
fn line_comment(input: RawSpan) -> ParseResult<'_, ()> {
    map(
        preceded(char(';'), many0(is_not("\r\n"))),
        |_| (), // throw the comment away
    )(input)
}

/// Parse everything that can go after a statement on a line: whitespace and
/// an optional comment. Also parses the line ending, or up to the end of file.
fn end_of_statement(input: RawSpan) -> ParseResult<'_, ()> {
    // Don't include the beginning whitespace in the context
    preceded(
        space0,
        context(
            "end of statement",
            map(terminated(opt(line_comment), eol_or_eof), |_| ()),
        ),
    )(input)
}

/// Parse a single line, up to and including either end of line or end of file.
fn line(input: RawSpan) -> ParseResult<'_, Option<SpanNode<Statement<Span>>>> {
    if input.fragment().is_empty() {
        // many0 fails if the parser consumers nothing, so we want to fail when
        // we normally would consume nothing
        Err(nom::Err::Error(VerboseError::from_error_kind(
            input,
            ErrorKind::Eof,
        )))
    } else {
        alt((
            // These contexts are for debugging only. Any error should have
            // a more precise context that can be shown to the user.
            context("empty line [debug]", map(end_of_statement, |_| None)),
            context(
                "line w/ statement [debug]",
                cut(map(
                    delimited(
                        space0,
                        context("statement", Statement::parse_node),
                        end_of_statement,
                    ),
                    Some,
                )),
            ),
        ))(input)
    }
}

/// Parse a full program
fn parse(
    input: &str,
) -> Result<Program<Span>, Vec<SourceErrorWrapper<CompileError>>> {
    match Program::parse(RawSpan::new(input)) {
        Ok((_, program)) => Ok(program),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            // Grab the first error in the chain that is a Context, which means
            // we labelled it ourselves. Everything else is generated by nom
            // which means it's useless.
            let (raw_span, context) = e
                .errors
                .iter()
                .filter_map(|err| match err {
                    (span, VerboseErrorKind::Context(context)) => {
                        Some((span, context))
                    }
                    _ => None,
                })
                .next()
                // This indicates we're missing a context() call somewhere
                .expect("No context errors available");

            Err(vec![SourceErrorWrapper::new(
                CompileError::Syntax { expected: context },
                // the actual fragment here is just the remaining source, so
                // it's not useful - just use the position from it
                Span::from_position(raw_span),
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

    /// Helper to make it a bit terser to create spans for tests
    fn span(
        offset: usize,
        length: usize,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Span {
        Span {
            offset,
            length,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(
            parse("LBL:\n\n\n").unwrap().body,
            vec![Node(
                Statement::Label(Node(
                    LabelDecl("LBL".into()),
                    span(0, 4, 1, 1, 1, 5)
                )),
                span(0, 4, 1, 1, 1, 5)
            )]
        );
        assert_eq!(
            parse("\n\nLBL:\n\n").unwrap().body,
            vec![Node(
                Statement::Label(Node(
                    LabelDecl("LBL".into()),
                    span(2, 4, 3, 1, 3, 5)
                )),
                span(2, 4, 3, 1, 3, 5)
            )]
        );
        assert_eq!(
            parse("  LBL:").unwrap().body,
            vec![Node(
                Statement::Label(Node(
                    LabelDecl("LBL".into()),
                    span(2, 4, 1, 3, 1, 7)
                )),
                span(2, 4, 1, 3, 1, 7)
            )]
        );
        assert_eq!(
            parse("  \n  LBL:").unwrap().body,
            vec![Node(
                Statement::Label(Node(
                    LabelDecl("LBL".into()),
                    span(5, 4, 2, 3, 2, 7)
                )),
                span(5, 4, 2, 3, 2, 7)
            )]
        );
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
                        span(17, 4, 2, 17, 2, 21)
                    )),
                    span(17, 4, 2, 17, 2, 21)
                ),
                Node(
                    Statement::Label(Node(
                        LabelDecl("LBL1".into()),
                        span(38, 5, 3, 17, 3, 22)
                    )),
                    span(38, 5, 3, 17, 3, 22)
                ),
                Node(
                    Statement::Label(Node(
                        LabelDecl("LBL_WITH_UNDERSCORE".into()),
                        span(60, 20, 4, 17, 4, 37)
                    )),
                    span(60, 20, 4, 17, 4, 37)
                ),
                Node(
                    Statement::Label(Node(
                        LabelDecl("1LBL".into()),
                        span(97, 5, 5, 17, 5, 22)
                    )),
                    span(97, 5, 5, 17, 5, 22)
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
                    Statement::Instruction(Node(
                        Instruction::Read(Node(
                            RegisterRef::User(0),
                            span(22, 3, 2, 22, 2, 25)
                        )),
                        span(17, 8, 2, 17, 2, 25)
                    )),
                    span(17, 8, 2, 17, 2, 25)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Write(Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(0),
                                span(48, 3, 3, 23, 3, 26)
                            )),
                            span(48, 3, 3, 23, 3, 26)
                        )),
                        span(42, 9, 3, 17, 3, 26)
                    )),
                    span(42, 9, 3, 17, 3, 26)
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
                SET RX1 RZR
                "
            )
            .unwrap()
            .body,
            vec![
                Node(
                    Statement::Instruction(Node(
                        Instruction::Set(
                            Node(
                                RegisterRef::User(1),
                                span(21, 3, 2, 21, 2, 24)
                            ),
                            Node(
                                ValueSource::Const(Node(
                                    4,
                                    span(25, 1, 2, 25, 2, 26)
                                )),
                                span(25, 1, 2, 25, 2, 26)
                            )
                        ),
                        span(17, 9, 2, 17, 2, 26)
                    )),
                    span(17, 9, 2, 17, 2, 26)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Set(
                            Node(
                                RegisterRef::User(1),
                                span(47, 3, 3, 21, 3, 24)
                            ),
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::InputLength,
                                    span(51, 3, 3, 25, 3, 28)
                                )),
                                span(51, 3, 3, 25, 3, 28)
                            )
                        ),
                        span(43, 11, 3, 17, 3, 28)
                    )),
                    span(43, 11, 3, 17, 3, 28)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Set(
                            Node(
                                RegisterRef::User(1),
                                span(75, 3, 4, 21, 4, 24)
                            ),
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::StackLength(0),
                                    span(79, 3, 4, 25, 4, 28)
                                )),
                                span(79, 3, 4, 25, 4, 28)
                            )
                        ),
                        span(71, 11, 4, 17, 4, 28)
                    )),
                    span(71, 11, 4, 17, 4, 28)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Set(
                            Node(
                                RegisterRef::User(1),
                                span(103, 3, 5, 21, 5, 24)
                            ),
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::Null,
                                    span(107, 3, 5, 25, 5, 28)
                                )),
                                span(107, 3, 5, 25, 5, 28)
                            )
                        ),
                        span(99, 11, 5, 17, 5, 28)
                    )),
                    span(99, 11, 5, 17, 5, 28)
                ),
            ]
        );
    }

    #[test]
    fn test_neg_literal() {
        assert_eq!(
            parse("Add RX1 -10").unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Add(
                        Node(RegisterRef::User(1), span(4, 3, 1, 5, 1, 8)),
                        Node(
                            ValueSource::Const(Node(
                                -10,
                                span(8, 3, 1, 9, 1, 12)
                            )),
                            span(8, 3, 1, 9, 1, 12)
                        )
                    ),
                    span(0, 11, 1, 1, 1, 12)
                )),
                span(0, 11, 1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_parse_lang_val_max() {
        let source = format!("Add RX1 {}", LangValue::MAX);
        assert_eq!(
            parse(&source).unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Add(
                        Node(RegisterRef::User(1), span(4, 3, 1, 5, 1, 8)),
                        Node(
                            ValueSource::Const(Node(
                                LangValue::MAX,
                                span(8, 10, 1, 9, 1, 19)
                            )),
                            span(8, 10, 1, 9, 1, 19)
                        )
                    ),
                    span(0, 18, 1, 1, 1, 19)
                )),
                span(0, 18, 1, 1, 1, 19)
            )]
        );
    }

    #[test]
    fn test_parse_lang_val_min() {
        let source = format!("Add RX1 {}", LangValue::min_value());
        assert_eq!(
            parse(&source).unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Add(
                        Node(RegisterRef::User(1), span(4, 3, 1, 5, 1, 8)),
                        Node(
                            ValueSource::Const(Node(
                                LangValue::min_value(),
                                span(8, 11, 1, 9, 1, 20)
                            )),
                            span(8, 11, 1, 9, 1, 20)
                        )
                    ),
                    span(0, 19, 1, 1, 1, 20)
                )),
                span(0, 19, 1, 1, 1, 20)
            )]
        );
    }

    #[test]
    fn test_add() {
        assert_eq!(
            parse("Add RX1 RX4").unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Add(
                        Node(RegisterRef::User(1), span(4, 3, 1, 5, 1, 8)),
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(4),
                                span(8, 3, 1, 9, 1, 12)
                            )),
                            span(8, 3, 1, 9, 1, 12)
                        )
                    ),
                    span(0, 11, 1, 1, 1, 12)
                )),
                span(0, 11, 1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            parse("Sub RX1 RX4").unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Sub(
                        Node(RegisterRef::User(1), span(4, 3, 1, 5, 1, 8)),
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(4),
                                span(8, 3, 1, 9, 1, 12)
                            )),
                            span(8, 3, 1, 9, 1, 12)
                        )
                    ),
                    span(0, 11, 1, 1, 1, 12)
                )),
                span(0, 11, 1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            parse("Mul RX1 RX4").unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Mul(
                        Node(RegisterRef::User(1), span(4, 3, 1, 5, 1, 8)),
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(4),
                                span(8, 3, 1, 9, 1, 12)
                            )),
                            span(8, 3, 1, 9, 1, 12)
                        )
                    ),
                    span(0, 11, 1, 1, 1, 12)
                )),
                span(0, 11, 1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_cmp() {
        assert_eq!(
            parse("CMP RX0 5 10").unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Cmp(
                        Node(RegisterRef::User(0), span(4, 3, 1, 5, 1, 8)),
                        Node(
                            ValueSource::Const(Node(
                                5,
                                span(8, 1, 1, 9, 1, 10)
                            )),
                            span(8, 1, 1, 9, 1, 10)
                        ),
                        Node(
                            ValueSource::Const(Node(
                                10,
                                span(10, 2, 1, 11, 1, 13)
                            )),
                            span(10, 2, 1, 11, 1, 13)
                        )
                    ),
                    span(0, 12, 1, 1, 1, 13)
                )),
                span(0, 12, 1, 1, 1, 13)
            )]
        );
    }

    #[test]
    fn test_push() {
        assert_eq!(
            parse("Push RX2 S4").unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Push(
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(2),
                                span(5, 3, 1, 6, 1, 9)
                            )),
                            span(5, 3, 1, 6, 1, 9)
                        ),
                        Node(StackRef(4), span(9, 2, 1, 10, 1, 12))
                    ),
                    span(0, 11, 1, 1, 1, 12)
                )),
                span(0, 11, 1, 1, 1, 12)
            )]
        );
    }

    #[test]
    fn test_pop() {
        assert_eq!(
            parse("Pop S4 RX2").unwrap().body,
            vec![Node(
                Statement::Instruction(Node(
                    Instruction::Pop(
                        Node(StackRef(4), span(4, 2, 1, 5, 1, 7)),
                        Node(RegisterRef::User(2), span(7, 3, 1, 8, 1, 11)),
                    ),
                    span(0, 10, 1, 1, 1, 11)
                )),
                span(0, 10, 1, 1, 1, 11)
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
                    Statement::Instruction(Node(
                        Instruction::Jmp(Node(
                            "LBL".into(),
                            span(21, 3, 2, 21, 2, 24)
                        )),
                        span(17, 7, 2, 17, 2, 24)
                    )),
                    span(17, 7, 2, 17, 2, 24)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Jez(
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::User(0),
                                    span(45, 3, 3, 21, 3, 24)
                                )),
                                span(45, 3, 3, 21, 3, 24)
                            ),
                            Node("LBL".into(), span(49, 3, 3, 25, 3, 28))
                        ),
                        span(41, 11, 3, 17, 3, 28)
                    )),
                    span(41, 11, 3, 17, 3, 28)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Jnz(
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::User(0),
                                    span(73, 3, 4, 21, 4, 24)
                                )),
                                span(73, 3, 4, 21, 4, 24)
                            ),
                            Node("LBL".into(), span(77, 3, 4, 25, 4, 28)),
                        ),
                        span(69, 11, 4, 17, 4, 28)
                    )),
                    span(69, 11, 4, 17, 4, 28)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Jlz(
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::User(0),
                                    span(101, 3, 5, 21, 5, 24)
                                )),
                                span(101, 3, 5, 21, 5, 24)
                            ),
                            Node("LBL".into(), span(105, 3, 5, 25, 5, 28))
                        ),
                        span(97, 11, 5, 17, 5, 28)
                    )),
                    span(97, 11, 5, 17, 5, 28)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Jgz(
                            Node(
                                ValueSource::Register(Node(
                                    RegisterRef::User(0),
                                    span(129, 3, 6, 21, 6, 24)
                                )),
                                span(129, 3, 6, 21, 6, 24)
                            ),
                            Node("LBL".into(), span(133, 3, 6, 25, 6, 28)),
                        ),
                        span(125, 11, 6, 17, 6, 28)
                    )),
                    span(125, 11, 6, 17, 6, 28)
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
                Statement::Instruction(Node(
                    Instruction::Add(
                        Node(RegisterRef::User(1), span(57, 3, 3, 21, 3, 24)),
                        Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(4),
                                span(61, 3, 3, 25, 3, 28)
                            )),
                            span(61, 3, 3, 25, 3, 28)
                        )
                    ),
                    span(53, 11, 3, 17, 3, 28)
                )),
                span(53, 11, 3, 17, 3, 28)
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
                    Statement::Instruction(Node(
                        Instruction::Read(Node(
                            RegisterRef::User(0),
                            span(53, 3, 3, 22, 3, 25)
                        )),
                        span(48, 8, 3, 17, 3, 25)
                    )),
                    span(48, 8, 3, 17, 3, 25)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Set(
                            Node(
                                RegisterRef::User(0),
                                span(108, 3, 5, 21, 5, 24)
                            ),
                            Node(
                                ValueSource::Const(Node(
                                    2,
                                    span(112, 1, 5, 25, 5, 26)
                                )),
                                span(112, 1, 5, 25, 5, 26)
                            )
                        ),
                        span(104, 9, 5, 17, 5, 26)
                    )),
                    span(104, 9, 5, 17, 5, 26)
                ),
                Node(
                    Statement::Instruction(Node(
                        Instruction::Write(Node(
                            ValueSource::Register(Node(
                                RegisterRef::User(0),
                                span(155, 3, 6, 23, 6, 26)
                            )),
                            span(155, 3, 6, 23, 6, 26)
                        )),
                        span(149, 9, 6, 17, 6, 26)
                    )),
                    span(149, 9, 6, 17, 6, 26)
                )
            ]
        );
    }
}
