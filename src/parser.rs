use nom::IResult;
use nom::{
    Parser,
    multi::{many0},
    bytes::{tag, is_a, take_until,},
    character::{complete::alpha1, complete::line_ending,},
    combinator::{value, opt,},
};

use crate::{ast};

pub fn parse<I>(input: I) -> IResult<I, (ast::Subprogram, Vec<ast::Subprogram>)>
where
    for<'parser> I: nom::Input + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>, 
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{
    println!("Parsing root");
    let (remaining, (root_knot, knots)) = (knot_body, many0(knot)).parse(input)?;
    let program = (root_knot, knots);

    Ok((remaining, program))
}


fn knot<I>(input: I) -> IResult<I, ast::Subprogram>
where
    for<'parser> I: nom::Input + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>, 
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{   
    println!("Parsing knot");
    value(ast::Subprogram::Knot, (knot_signature, knot_body)).parse(input)
}

fn knot_body<I>(input: I) -> IResult<I, ast::Subprogram> 
where
    for<'parser> I: nom::Input + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>, 
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{ 
    println!("Parsing body");
    value(ast::Subprogram::Knot, take_until("==")).parse(input) 
}

fn knot_signature<I>(input: I) -> IResult<I, ast::Subprogram> 
where
    for<'parser> I: nom::Input + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>, 
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{ 
    println!("Parsing signature");
    value(ast::Subprogram::Knot, (tag("=="), opt(is_a("=")), identifier, opt(is_a("=")), line_ending)).parse(input)
}

fn identifier<I>(input: I) -> IResult<I, ast::Subprogram> 
where
    for<'parser> I: nom::Input + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>, 
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item>, 
{
    //TODO: More permissive identifier
    value(ast::Subprogram::Knot, alpha1).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_knots_with_root() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, parsed) = parse(include_str!("../tests/knots_with_root.ink"))?;

        assert_eq!(parsed, (ast::Subprogram::Knot, vec![ast::Subprogram::Knot;2]), "Invalid parse.");
        assert!(unparsed.is_empty(), "Incomplete parse. Remaining text: {}", unparsed);
        Ok(())
    }
}
