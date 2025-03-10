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

pub fn parse<I>(input: I) -> IResult<I, ((ast::Callable, I), Vec<(ast::Callable, I)>)>
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item>,
{
    let (remaining, (root_knot, knots, _)) = (knot_body, many0(knot), alt((line_ending, eof))).parse(input)?;
    let program = (
        (
            ast::Callable{ 
                ty: ast::Subprogram::Knot, 
                name: "__root".into(),
                parameters: vec![],
            },
            root_knot
        ), 
        knots
    );

    Ok((remaining, program))
}


fn knot<I>(input: I) -> IResult<I, (ast::Callable, I)>
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item>,
{   
    let (rem, (name, parameters)) = knot_signature.parse(input)?;
    let (rem, body) = knot_body.parse(rem)?;

    Ok((rem, (ast::Callable{name: name.into(), parameters, ty: ast::Subprogram::Knot}, body)))
}

fn knot_body<I, T>(input: I) -> IResult<I, I> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str> + nom::FindSubstring<T>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{ 
    let (rem, body) = match take_until("==").parse(input.clone()) {
        Ok((rem, body)) => {
            peek(recognize(knot_signature)).parse(rem.clone())?;
            (rem, body)
        },
        nom::IResult::Err(nom::Err::Incomplete(_)) => {
            input.take_split(input.input_len()-1)
        },
        err => err?
    };
    
    Ok((rem, body))
}

fn knot_signature<I>(input: I) -> IResult<I, (ast::Identifier, Vec<ast::Parameter>)> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{ 
    let (rem, _) = (space0, tag("=="), opt(is_a("=")), space0).parse(input)?;
    let (rem, name) = identifier.parse(rem)?;
    let (rem, _) = (space0, opt(is_a("=")), line_ending).parse(rem)?;
    Ok((rem, (name, vec![]/*TODO: Parse parameters*/)))
}

fn identifier<I>(input: I) -> IResult<I, ast::Identifier> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item>, 
{
    //TODO: More permissive identifier
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
                (ast::Callable{ty: ast::Subprogram::Knot, name: root_name, ..}, root_body), 
            [
                (ast::Callable{ty: ast::Subprogram::Knot, name: k1_name, ..}, _),
                (ast::Callable{ty: ast::Subprogram::Knot, name: k2_name, ..}, _),
            ])  if root_body.trim() != "" && (k1_name == "K1" && k2_name == "K2")=> {},
            _ => { panic!("Invalid parse.\nRemaining: \n{}\n---", unparsed); }
        };

        Ok(())
    }

    #[test]
    fn parse_knots_without_root() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, (root, knots)) = parse(include_str!("../tests/knots_without_root.ink"))?;

        match eof::<&str,nom::error::Error<&str>>.parse(unparsed) {
            Ok(_) => {},
            _ => assert!(false, "Incomplete parse. Remaining text: {}:'{}'", unparsed.input_len(), unparsed),
        }

        match (root, knots.as_slice()) {
            (
                (ast::Callable{ty: ast::Subprogram::Knot, name: root_name, ..}, root_body), 
            [
                (ast::Callable{ty: ast::Subprogram::Knot, name: k1_name, ..}, _),
                (ast::Callable{ty: ast::Subprogram::Knot, name: k2_name, ..}, _),
            ])  if root_body.trim() == "" && (k1_name == "K1" && k2_name == "K2")=> {},
            _ => { panic!("Invalid parse.\nRemaining: \n{}\n---", unparsed); }
        };

        Ok(())
    }
}
