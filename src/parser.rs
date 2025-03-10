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

pub fn parse<I>(input: I) -> IResult<I, (ast::Subprogram, Vec<ast::Subprogram>)>
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
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
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{   
    println!("Parsing knot");
    let (rem, _signature) = knot_signature.parse(input)?;
    let (rem, _body) = knot_body.parse(rem)?;

    //let (rem, _body) = knot_body.parse(rem)?;
    println!("End of knot");
    Ok((rem, ast::Subprogram::Knot))
    //value(ast::Subprogram::Knot, (knot_signature, knot_body)).parse(input)
}

fn knot_body<I, T>(input: I) -> IResult<I, ast::Subprogram> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str> + nom::FindSubstring<T>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{ 
    println!("Parsing body");
    //value(ast::Subprogram::Knot, many_till( recognize(      (anychar, anychar)  ),  peek(alt(    (tag("=="), eof)                ))  )).parse(input) 

    //println!("\n----\n{}\n<<<<\n", chunk.iter().map(nom::Input::iter_elements).flatten().map(nom::AsChar::as_char).collect::<String>());
    //println!("End body");
    //Ok((rem, ast::Subprogram::Knot))
    let (rem, body) = match take_until("==").parse(input.clone()) {
        Ok((rem, body)) => {
            peek(recognize(knot_signature)).parse(rem)?
        },
        nom::IResult::Err(nom::Err::Incomplete(_)) => {
            println!("\teof!");
            input.take_split(input.input_len()-1)
        },
        err => err?
    };
    println!("\n>>>>\n{}\n>>>>\n", body.iter_elements().map(nom::AsChar::as_char).collect::<String>());
    println!("\n<<<<\n{}\n<<<<\n", rem.iter_elements().map(nom::AsChar::as_char).collect::<String>());
    
    Ok((rem, ast::Subprogram::Knot))
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
    //value(ast::Subprogram::Knot, (space0, opt(is_a("=")), line_ending)).parse(rem)
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

    #[test]
    fn parse_knots_with_root() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, parsed) = parse(include_str!("../tests/knots_with_root.ink"))?;

        assert_eq!(parsed, (ast::Subprogram::Knot, vec![ast::Subprogram::Knot;2]), "Invalid parse.\nRemaining: \n{}\n---", unparsed);
        assert!(unparsed.is_empty(), "Incomplete parse. Remaining text: {}", unparsed);
        Ok(())
    }
}
