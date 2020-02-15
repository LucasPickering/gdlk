use crate::{
    ast::{
        source::{Program, Statement},
        Jump, Label, LangValue, Operator, RegisterRef, StackIdentifier,
        UserRegisterIdentifier, ValueSource,
    },
    consts::{REG_INPUT_LEN, REG_STACK_LEN_PREFIX, REG_USER_PREFIX},
    error::CompileError,
    Compiler,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_while1},
    character::complete::{char, digit1, line_ending, space0, space1},
    combinator::{all_consuming, cut, map, map_res, opt, recognize},
    error::{
        context, convert_error, ErrorKind, ParseError, VerboseError,
        VerboseErrorKind,
    },
    lib::std::ops::RangeTo,
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    AsChar, Compare, IResult, InputTake, InputTakeAtPosition, Slice,
};
use std::iter;

type ParseResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

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

/// Parses a register identifer, something like "RX0". Does not parse any
/// whitespace around it.
fn reg_ident(input: &str) -> ParseResult<'_, RegisterRef> {
    let (input, val) = context(
        "Register",
        alt((
            // "RLI" => RegisterRef::InputLength
            map(tag_no_case(REG_INPUT_LEN), |_| RegisterRef::InputLength),
            // "RSx" => RegisterRef::StackLength(x)
            preceded(
                tag_no_case(REG_STACK_LEN_PREFIX),
                cut(map_res(digit1, |s: &str| {
                    s.parse::<StackIdentifier>().map(RegisterRef::StackLength)
                })),
            ),
            // "RXx" => RegisterRef::User(x)
            preceded(
                tag_no_case(REG_USER_PREFIX),
                cut(map_res(digit1, |s: &str| {
                    s.parse::<UserRegisterIdentifier>().map(RegisterRef::User)
                })),
            ),
        )),
    )(input)?;
    Ok((input, val))
}

/// Parses a stack identifier, like "S1", not including surrounding  whitespace.
fn stack_ident(input: &str) -> ParseResult<'_, StackIdentifier> {
    preceded(
        tag_no_case("S"),
        map_res(digit1, |s: &str| s.parse::<StackIdentifier>()),
    )(input)
}

/// Parses a `LangValue`, like "10" or "-3", not including any surrounding
/// whitespace.
fn lang_value(input: &str) -> ParseResult<'_, LangValue> {
    map_res(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
        s.parse::<LangValue>()
    })(input)
}

/// Parses either a `LangValue` or `Register`.
fn value_source(input: &str) -> ParseResult<'_, ValueSource> {
    alt((
        // "1" => ValueSource::Const(1)
        map(lang_value, ValueSource::Const),
        // "RX1" => ValueSource::Register(1)
        map(reg_ident, ValueSource::Register),
    ))(input)
}

/// Parses a label (either declaration or usage), NOT including the trailing
/// colon.
fn label(input: &str) -> ParseResult<'_, Label> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        Label::from,
    )(input)
}

/// Matches a label statement (i.e. label declaration).
fn label_stmt(input: &str) -> ParseResult<'_, Statement> {
    map(terminated(label, tag(":")), Statement::Label)(input)
}

fn operator_stmt(input: &str) -> ParseResult<'_, Statement> {
    map(
        alt((
            tag_with_args("READ", one_arg(reg_ident), Operator::Read),
            tag_with_args("WRITE", one_arg(value_source), Operator::Write),
            tag_with_args(
                "SET",
                two_args(reg_ident, value_source),
                |(dst, src)| Operator::Set(dst, src),
            ),
            tag_with_args(
                "ADD",
                two_args(reg_ident, value_source),
                |(dst, src)| Operator::Add(dst, src),
            ),
            tag_with_args(
                "SUB",
                two_args(reg_ident, value_source),
                |(dst, src)| Operator::Sub(dst, src),
            ),
            tag_with_args(
                "MUL",
                two_args(reg_ident, value_source),
                |(dst, src)| Operator::Mul(dst, src),
            ),
            tag_with_args(
                "CMP",
                three_args(reg_ident, value_source, value_source),
                |(dst, src_1, src_2)| Operator::Cmp(dst, src_1, src_2),
            ),
            tag_with_args(
                "PUSH",
                two_args(value_source, stack_ident),
                |(src, stack)| Operator::Push(src, stack),
            ),
            tag_with_args(
                "POP",
                two_args(stack_ident, reg_ident),
                |(stack, dst)| Operator::Pop(stack, dst),
            ),
        )),
        Statement::Operator,
    )(input)
}

fn jump_stmt(input: &str) -> ParseResult<'_, Statement> {
    alt((
        tag_with_args("JMP", one_arg(label), |l| Statement::Jump(Jump::Jmp, l)),
        tag_with_args("JEZ", two_args(value_source, label), |(src, l)| {
            Statement::Jump(Jump::Jez(src), l)
        }),
        tag_with_args("JNZ", two_args(value_source, label), |(src, l)| {
            Statement::Jump(Jump::Jnz(src), l)
        }),
        tag_with_args("JGZ", two_args(value_source, label), |(src, l)| {
            Statement::Jump(Jump::Jgz(src), l)
        }),
        tag_with_args("JLZ", two_args(value_source, label), |(src, l)| {
            Statement::Jump(Jump::Jlz(src), l)
        }),
    ))(input)
}

/// Parses a line comment, which starts with a ; and runs to the end of the
/// line. This terminates at the line ending, but does _not_ consume it.
fn line_comment(input: &str) -> ParseResult<'_, &str> {
    map(
        context("Comment", preceded(char(';'), many0(is_not("\r\n")))),
        |_| "", // throw the comment away
    )(input)
}

/// Parses one source statement, not including surrounding whitespace.
fn statement(input: &str) -> ParseResult<'_, Statement> {
    context("Statement", alt((label_stmt, operator_stmt, jump_stmt)))(input)
}

/// Parse a single line, not including the line ending.
fn line(input: &str) -> ParseResult<'_, Option<Statement>> {
    context(
        "Line",
        delimited(space0, opt(statement), tuple((space0, opt(line_comment)))),
    )(input)
}

/// Parse a full program
fn parse_gdlk(input: &str) -> ParseResult<'_, Vec<Statement>> {
    map(
        // separated_list doesn't work properly so we have to do this
        all_consuming(tuple((many0(terminated(line, line_ending)), opt(line)))),
        // filter out None lines
        |(lines, last_line)| {
            lines
                .into_iter()
                .chain(iter::once(last_line.unwrap_or(None)))
                .filter_map(std::convert::identity)
                .collect()
        },
    )(input)
}

fn parse(input: &str) -> Result<Vec<Statement>, CompileError> {
    match parse_gdlk(input) {
        Ok((_, body)) => Ok(body),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            match e.errors.as_slice() {
                [(substring, VerboseErrorKind::Nom(ErrorKind::Eof)), ..] => {
                    // If the error is EOF that means there was remaining
                    // input that was not parsed
                    // so they put in a bad keyword
                    // TODO: need to make this custom error look more like
                    // how convert_error outputs
                    Err(CompileError::ParseError(format!(
                        "Invalid keyword: {}",
                        substring
                    )))
                }
                _ => Err(CompileError::ParseError(convert_error(&input, e))),
            }
        }
        Err(nom::Err::Incomplete(_needed)) => {
            // TODO: better ass
            Err(CompileError::ParseError("ass".to_string()))
        }
    }
}

impl Compiler<String> {
    /// Parses source code from the given input, into an abstract syntax tree.
    pub fn parse(self) -> Result<Compiler<Program>, CompileError> {
        parse(&self.0).map(|stmts| Compiler(Program { body: stmts }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        assert_eq!(line(""), Ok(("", None)));
        assert_eq!(line("    "), Ok(("", None)));
        assert_eq!(line(" ; comment"), Ok(("", None)));
        assert_eq!(
            line("  READ RX0 ;comment"),
            Ok((
                "",
                Some(Statement::Operator(Operator::Read(RegisterRef::User(0))))
            ))
        );
        assert_eq!(line(" ; comment\nMORE"), Ok(("\nMORE", None)));
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(parse(""), Ok(vec![]));
        assert_eq!(parse("\n\n\n"), Ok(vec![]));
        assert_eq!(parse("  "), Ok(vec![]));
        assert_eq!(parse("  \n  "), Ok(vec![]));
    }

    #[test]
    fn test_parse_labels() {
        assert_eq!(
            parse_gdlk(
                "
                LBL:
                LBL1:
                LBL_WITH_UNDERSCORE:
                1LBL:
                "
            ),
            Ok((
                "",
                vec![
                    Statement::Label("LBL".into()),
                    Statement::Label("LBL1".into()),
                    Statement::Label("LBL_WITH_UNDERSCORE".into()),
                    Statement::Label("1LBL".into())
                ]
            ))
        )
    }

    #[test]
    fn test_parse_read_write() {
        assert_eq!(
            parse_gdlk(
                "
                ReAd RX0
                WrItE RX0
                "
            ),
            Ok((
                "",
                vec![
                    Statement::Operator(Operator::Read(RegisterRef::User(0))),
                    Statement::Operator(Operator::Write(
                        ValueSource::Register(RegisterRef::User(0))
                    ))
                ]
            ))
        )
    }

    #[test]
    fn test_set_and_registers() {
        assert_eq!(
            parse_gdlk(
                "
                Set RX1 4
                SET RX1 RLI
                SET RX1 RS0
                "
            ),
            Ok((
                "",
                vec![
                    Statement::Operator(Operator::Set(
                        RegisterRef::User(1),
                        ValueSource::Const(4)
                    )),
                    Statement::Operator(Operator::Set(
                        RegisterRef::User(1),
                        ValueSource::Register(RegisterRef::InputLength),
                    )),
                    Statement::Operator(Operator::Set(
                        RegisterRef::User(1),
                        ValueSource::Register(RegisterRef::StackLength(0)),
                    )),
                ]
            ))
        )
    }

    #[test]
    fn test_add() {
        assert_eq!(
            parse_gdlk("Add RX1 RX4"),
            Ok((
                "",
                vec![Statement::Operator(Operator::Add(
                    RegisterRef::User(1),
                    ValueSource::Register(RegisterRef::User(4))
                ))]
            ))
        )
    }

    #[test]
    fn test_neg_literal() {
        assert_eq!(
            parse_gdlk("Add RX1 -10"),
            Ok((
                "",
                vec![Statement::Operator(Operator::Add(
                    RegisterRef::User(1),
                    ValueSource::Const(-10)
                ))]
            ))
        )
    }

    #[test]
    fn test_parse_lang_val_max() {
        let source = format!("Add RX1 {}", std::i32::MAX);
        assert_eq!(
            parse_gdlk(source.as_str()),
            Ok((
                "",
                vec![Statement::Operator(Operator::Add(
                    RegisterRef::User(1),
                    ValueSource::Const(std::i32::MAX)
                ))]
            ))
        )
    }

    #[test]
    fn test_parse_lang_val_min() {
        let source = format!("Add RX1 {}", std::i32::MIN);
        assert_eq!(
            parse_gdlk(source.as_str()),
            Ok((
                "",
                vec![Statement::Operator(Operator::Add(
                    RegisterRef::User(1),
                    ValueSource::Const(std::i32::MIN)
                ))]
            ))
        )
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            parse_gdlk("Sub RX1 RX4"),
            Ok((
                "",
                vec![Statement::Operator(Operator::Sub(
                    RegisterRef::User(1),
                    ValueSource::Register(RegisterRef::User(4))
                ))]
            ))
        )
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            parse_gdlk("Mul rx1 rx0"),
            Ok((
                "",
                vec![Statement::Operator(Operator::Mul(
                    RegisterRef::User(1),
                    ValueSource::Register(RegisterRef::User(0))
                ))]
            ))
        )
    }

    #[test]
    fn test_cmp() {
        assert_eq!(
            parse_gdlk("CMP RX0 5 10"),
            Ok((
                "",
                vec![Statement::Operator(Operator::Cmp(
                    RegisterRef::User(0),
                    ValueSource::Const(5),
                    ValueSource::Const(10),
                ))]
            ))
        )
    }

    #[test]
    fn test_push() {
        assert_eq!(
            parse_gdlk("Push RX2 S4"),
            Ok((
                "",
                vec![Statement::Operator(Operator::Push(
                    ValueSource::Register(RegisterRef::User(2)),
                    4
                ))]
            ))
        )
    }

    #[test]
    fn test_pop() {
        assert_eq!(
            parse_gdlk("Pop S4 RX2"),
            Ok((
                "",
                vec![Statement::Operator(Operator::Pop(
                    4,
                    RegisterRef::User(2)
                ))]
            ))
        )
    }

    #[test]
    fn test_jumps() {
        assert_eq!(
            parse_gdlk(
                "JMP LBL
                JEZ RX0 LBL
                JNZ RX0 LBL
                JLZ RX0 LBL
                JGZ RX0 LBL
                "
            ),
            Ok((
                "",
                vec![
                    Statement::Jump(Jump::Jmp, "LBL".into()),
                    Statement::Jump(
                        Jump::Jez(ValueSource::Register(RegisterRef::User(0))),
                        "LBL".into()
                    ),
                    Statement::Jump(
                        Jump::Jnz(ValueSource::Register(RegisterRef::User(0))),
                        "LBL".into()
                    ),
                    Statement::Jump(
                        Jump::Jlz(ValueSource::Register(RegisterRef::User(0))),
                        "LBL".into()
                    ),
                    Statement::Jump(
                        Jump::Jgz(ValueSource::Register(RegisterRef::User(0))),
                        "LBL".into()
                    ),
                ]
            ))
        )
    }

    #[test]
    fn test_comments() {
        assert_eq!(
            parse_gdlk("; comment over here\n Add RX1 RX4 ; comment here\n"),
            Ok((
                "",
                vec![Statement::Operator(Operator::Add(
                    RegisterRef::User(1),
                    ValueSource::Register(RegisterRef::User(4))
                ))]
            ))
        )
    }

    #[test]
    fn test_parse_simple_file() {
        assert_eq!(
            parse_gdlk(
                ";comment start
            Read RX0
            ; comment poop
            Set RX0 2 ;comment more poop
            Write RX0
            Read RX1
            Set RX1 3
            Write RX1
            Read RX2
            Set RX2 4
            Write RX2
            ; comment pog
        "
            ),
            Ok((
                "",
                vec![
                    Statement::Operator(Operator::Read(RegisterRef::User(0))),
                    Statement::Operator(Operator::Set(
                        RegisterRef::User(0),
                        ValueSource::Const(2)
                    )),
                    Statement::Operator(Operator::Write(
                        ValueSource::Register(RegisterRef::User(0))
                    )),
                    Statement::Operator(Operator::Read(RegisterRef::User(1))),
                    Statement::Operator(Operator::Set(
                        RegisterRef::User(1),
                        ValueSource::Const(3)
                    )),
                    Statement::Operator(Operator::Write(
                        ValueSource::Register(RegisterRef::User(1))
                    )),
                    Statement::Operator(Operator::Read(RegisterRef::User(2))),
                    Statement::Operator(Operator::Set(
                        RegisterRef::User(2),
                        ValueSource::Const(4)
                    )),
                    Statement::Operator(Operator::Write(
                        ValueSource::Register(RegisterRef::User(2))
                    ))
                ]
            ))
        )
    }
}
