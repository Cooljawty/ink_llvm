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
    println!("Parsing knot");
    value(ast::Subprogram::Knot, (knot_signature, knot_body)).parse(input)
}

fn knot_body<I>(input: I) -> IResult<I, ast::Subprogram> 
where
    I: nom::Input + for<'parser> nom::FindSubstring<&'parser str>, 
{ 
    println!("Parsing body");
    value(ast::Subprogram::Knot, take_until("==")).parse(input) 
}

fn knot_signature<I>(input: I) -> IResult<I, ast::Subprogram> 
where
    I: nom::Input, 
{ 
    Ok((input, ast::Subprogram::Knot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_knots_with_root() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, parsed) = parse(include_str!("../tests/knots_with_root.ink"))?;

        assert_eq!(parsed, (ast::Subprogram::Knot, vec![ast::Subprogram::Knot;2]), 
            "Invalid parse. parsed as {:?}", parsed);
        assert!(unparsed.is_empty(), "Incomplete parse. Remaining text: {}", unparsed);
        Ok(())
    }
}
