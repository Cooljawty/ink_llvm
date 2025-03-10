use nom::IResult;

#[allow(unused_imports)]
use nom::{
    Parser,
    multi::{many0, many_till, fold_many0},
    bytes::{tag, is_a, take_until,take_till},
    character::{
        anychar,
        complete::{
            space0,
            alpha1, alphanumeric0,
            line_ending, not_line_ending,
        },
    },
    combinator::{value, opt, eof, not, peek, recognize,success,all_consuming,},
    branch::{alt,},
};

use crate::{ast};

pub fn parse<I>(input: I) -> IResult<I, ((ast::Subprogram, I), Vec<(ast::Subprogram, I)>)>
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{
    println!("Parsing root");
    let (remaining, (root_knot, knots, _)) = (knot_body, many0(knot), alt((line_ending, eof))).parse(input)?;
    let program = ((ast::Subprogram::Knot, root_knot), knots);

    Ok((remaining, program))
}


fn knot<I>(input: I) -> IResult<I, (ast::Subprogram, I)>
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{   
    println!("Parsing knot");
    let (rem, _signature) = knot_signature.parse(input)?;
    let (rem, body) = knot_body.parse(rem)?;

    Ok((rem, (ast::Subprogram::Knot, body)))
}

fn knot_body<I, T>(input: I) -> IResult<I, I> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str> + nom::FindSubstring<T>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{ 
    println!("Parsing body");
    let (rem, body) = match take_until("==").parse(input.clone()) {
        Ok((rem, body)) => {
            peek(recognize(knot_signature)).parse(rem.clone())?;
            (rem, body)
        },
        nom::IResult::Err(nom::Err::Incomplete(_)) => {
            println!("\teof!");
            input.take_split(input.input_len()-1)
        },
        err => err?
    };
    
    Ok((rem, body))
}

fn knot_signature<I>(input: I) -> IResult<I, ast::Subprogram> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{ 
    println!("Parsing signature");
    let (rem, _) = (space0, tag("=="), opt(is_a("=")), space0).parse(input)?;
    let (rem, name) = identifier.parse(rem)?;
    println!("\tname: {}", name);
    let (rem, _) = (space0, opt(is_a("=")), line_ending).parse(rem)?;
    println!("End signature");
    Ok((rem, ast::Subprogram::Knot))
}

fn identifier<I>(input: I) -> IResult<I, String> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item>, 
{
    //TODO: More permissive identifier
    println!("Parsing identifier");
    let (rem, (first, rest)) = (alpha1, alphanumeric0).parse(input)?;

    use nom::AsChar;
    let name = first.iter_elements().chain(rest.iter_elements()).map(|c| c.as_char()).collect();

    Ok((rem, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::Input;

    #[test]
    fn parse_knots_with_root() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, (root, knots)) = parse(include_str!("../tests/knots_with_root.ink"))?;

        match eof::<&str,nom::error::Error<&str>>.parse(unparsed) {
            Ok(_) => {},
            _ => assert!(false, "Incomplete parse. Remaining text: {}:'{}'", unparsed.input_len(), unparsed),
        }

        match (root, knots.as_slice()) {
            (
                (ast::Subprogram::Knot, _), 
            [
                (ast::Subprogram::Knot, _),
                (ast::Subprogram::Knot, _),
            ]) => {},
            _ => { panic!("Invalid parse.\nRemaining: \n{}\n---", unparsed); }
        };

        Ok(())
    }

    #[test]
    fn parse_knots_without_root() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, (root, knots)) = parse(include_str!("../tests/knots_with_root.ink"))?;

        match eof::<&str,nom::error::Error<&str>>.parse(unparsed) {
            Ok(_) => {},
            _ => assert!(false, "Incomplete parse. Remaining text: {}:'{}'", unparsed.input_len(), unparsed),
        }

        match (root, knots.as_slice()) {
            (
                (ast::Subprogram::Knot, ""), 
            [
                (ast::Subprogram::Knot, _),
                (ast::Subprogram::Knot, _),
            ]) => {},
            _ => { panic!("Invalid parse.\nRemaining: \n{}\n---", unparsed); }
        };

        Ok(())
    }
}
