use nom::IResult;

/* Debug printer
use nom::AsChar;
println!("\n>>>>\n{}\n>>>>\n", input.iter_elements().map(|c| c.as_char()).collect::<String>().as_str().lines().next().unwrap());
println!("\n<<<<\n{}\n<<<<\n", rem.iter_elements().map(|c| c.as_char()).collect::<String>().as_str().lines().next().unwrap());
*/

#[allow(unused_imports)]
use nom::{
    Parser,
    multi::{many0, many_till, fold_many0,separated_list0,},
    bytes::{tag, is_a, take_until,take_till},
    character::{
        anychar,one_of,
        complete::{
            space0,
            alpha1, alphanumeric1,
            line_ending, not_line_ending,
        },
    },
    combinator::{value, opt, eof, not, peek, recognize,success,all_consuming,flat_map,verify},
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
    let (rem, (signature, body)) = (knot_signature, knot_body).parse(input)?;

    Ok((rem, (signature, body)))
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

fn knot_signature<I>(input: I) -> IResult<I, ast::Callable> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item> 
{ 


    let (rem, (_, (name, _, parameters), _)) =
    (
        (space0, tag("=="), opt(is_a("=")), space0),
        (identifier, space0, opt(parameter_list)),
        (space0, opt(is_a("=")), line_ending)
    ).parse(input)?;
    
    Ok((rem, ast::Callable{
        name: name.into(), 
        parameters: parameters.unwrap_or(vec![]), 
        ty: ast::Subprogram::Knot
    }))
}

fn identifier<I>(input: I) -> IResult<I, ast::Identifier> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item>, 
{
    let (rem, name) = verify(
        //TODO: Add parsers for non-ASCII characters
        recognize(many0(alt((alphanumeric1, tag("_"))))),
        |n: &I| n.iter_elements()
            .nth(0)
            .expect("Parsed identifier as empty string?!")
            .is_alpha(),
        ).parse(input)?;

    use nom::AsChar;
    let name = name.iter_elements().map(|c| c.as_char()).collect();

    Ok((rem, name))
}

fn parameter_list<I>(input: I) -> IResult<I, Vec<ast::Parameter>> 
where
	for<'parser> I: nom::Input + nom::Offset + nom::Compare<&'parser str> + nom::FindSubstring<&'parser str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'parser> &'parser str: nom::FindToken<<I as nom::Input>::Item>, 
{
    let (rem, (_, param_list, _)) = (
        (tag("("), space0),
            separated_list0(
                (space0, tag(","), space0),
                (
                    opt(tag("ref")), space0, opt(tag("->")), 
                    space0, 
                    identifier 
                ),
            ),
        (space0, tag(")")),
    ).parse(input)?;

    let param_list = param_list.iter().map(|(is_ref, _, is_divert, _, name)| { 
        ast::Parameter{ 
            name: name.to_string(), 
            refrence: is_ref.is_some(), 
            is_divert: is_divert.is_some() 
        } 
    }).collect();

    Ok((rem, param_list))
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
            ])  if root_body.trim() != "" && (root_name == "__root" && k1_name == "K1" && k2_name == "K2")=> {},
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
            ])  if root_body.trim() == "" && (root_name == "__root" && k1_name == "K1" && k2_name == "K2")=> {},
            _ => { panic!("Invalid parse.\nRemaining: \n{}\n---", unparsed); }
        };

        Ok(())
    }

    #[test]
    fn parse_knot_with_parameters() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, knots) = nom::multi::many1(knot).parse(include_str!("../tests/knots_with_parameters.ink"))?;
        match knots.as_slice() {
            [
                (ast::Callable{ty: ast::Subprogram::Knot, parameters: k1_parameters, ..}, _),
                (ast::Callable{ty: ast::Subprogram::Knot, parameters: k2_parameters, ..}, _),
                (ast::Callable{ty: ast::Subprogram::Knot, parameters: k3_parameters, ..}, _),
                (ast::Callable{ty: ast::Subprogram::Knot, parameters: k4_parameters, ..}, _),
                (ast::Callable{ty: ast::Subprogram::Knot, parameters: k5_parameters, ..}, _),
                (ast::Callable{ty: ast::Subprogram::Knot, parameters: k6_parameters, ..}, _),
            ] => {
                assert!(matches!(k1_parameters.as_slice(), []), "Expected 0 arguments");
                assert!(matches!(k2_parameters.as_slice(), [ast::Parameter{..}]), "Expected 1 argument");
                assert!(matches!(k3_parameters.as_slice(), [ast::Parameter{..}, ast::Parameter{..}, ast::Parameter{..}]), "Expected 3 arguments");
                assert!(matches!(k4_parameters.as_slice(), [ast::Parameter{refrence:  true,  is_divert: false, ..}]), "Expected 1 argument by refrence");
                assert!(matches!(k5_parameters.as_slice(), [ast::Parameter{refrence:  false, is_divert: true,  ..}]), "Expected 1 divert argument by value");
                assert!(matches!(k6_parameters.as_slice(), [ast::Parameter{refrence:  true,  is_divert: true,  ..}]), "Expected 1 divert argument by refrence");
            },
            _ => { panic!("Invalid parse.\nRemaining: \n{}\n---", unparsed); }
        };

        Ok(())
    }
}
