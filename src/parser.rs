use nom::IResult;
use nom::{
    Parser,
    multi::{many0},
    bytes::{take_until,},
    combinator::{value,},
};

use crate::{ast};


pub fn parse<I>(input: I) -> IResult<I, (ast::Subprogram, Vec<ast::Subprogram>)>
where
    I: nom::Input + for<'parser> nom::FindSubstring<&'parser str>, 
{
    let (remaining, (root_knot, knots)) = (knot_body, many0(knot)).parse(input)?;
    let program = (root_knot, knots);

    Ok((remaining, program))
}


fn knot<I>(input: I) -> IResult<I, ast::Subprogram>
where
    I: nom::Input + for<'parser> nom::FindSubstring<&'parser str>, 
{   
    value(ast::Subprogram::Knot, (knot_signature, knot_body)).parse(input)
}

fn knot_body<I>(input: I) -> IResult<I, ast::Subprogram> 
where
    I: nom::Input + for<'parser> nom::FindSubstring<&'parser str>, 
{ value(ast::Subprogram::Knot, take_until("==")).parse(input) }

fn knot_signature<I>(input: I) -> IResult<I, ast::Subprogram> 
where
    I: nom::Input, 
{ todo!() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() -> Result<(), >{
        let result = Parser::parse();

        assert_eq!(result, /*AST*/);

        OK(())
    }
}
