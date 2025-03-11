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

#[allow(dead_code)]
#[derive(Debug)]
pub struct Knot<I> {
    signature: ast::Callable, 
    root: Stitch<I>, 
    body: Vec<Stitch<I>>,
}
#[allow(dead_code)]
#[derive(Debug)]
pub struct Stitch<I> {
    signature: ast::Callable,
    body: I
}

//type Program = (Vec<Knot>)
pub fn parse<I>(input: I) -> IResult<I, (Knot<I>, Vec<Knot<I>>) >
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    let (remaining, ((root_root_stitch, root_stitches), knots, _)) = (knot_body, many0(knot), alt((line_ending, eof))).parse(input)?;

    let root_signature = ast::Callable{ ty: ast::Subprogram::Knot, name: "__root".into(), parameters: vec![], };

    let program = (
        Knot {
            signature: root_signature, 
            root:      root_root_stitch,
            body:      root_stitches
        },
        knots
    );
    Ok((remaining, program))
}


fn knot<I>(input: I) -> IResult<I, Knot<I>>
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{   
    println!("Parsing knot");
    let (rem, (signature, (root_stitch, stitches))) = (knot_signature, knot_body).parse(input)?;
    println!("End knot");
    Ok ((rem, Knot {
            signature: signature, 
            root:      root_stitch,
            body:      stitches
    }))
}

fn knot_body<I>(input: I) -> IResult<I, (Stitch<I>, Vec<Stitch<I>>)> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    println!("Parsing knot body");
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
    
    println!("Took body input");
    let (_, (body, stitches)) = (stitch_body, many0(stitch)).parse(body)?;

    let root_stitch = Stitch {
        signature: ast::Callable{ 
            ty: ast::Subprogram::Stitch, 
            name: "__root".into(), 
            parameters: vec![], 
        }, 
        body,
    };
    println!("end knot body");
    Ok((rem, (root_stitch, stitches)))
}

fn stitch<I>(input: I) -> IResult<I, Stitch<I>>
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{   
    println!("Parsing stitch");
    let (rem, (signature, body)) = (stitch_signature, stitch_body).parse(input)?;
    println!("End stitch");
    Ok((rem, Stitch{signature, body}))
}

fn stitch_body<I>(input: I) -> IResult<I, I> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{ 
    println!("Parsing stitch body");
use nom::AsChar;
println!("\n>>>>\n{}\n>>>>\n", input.iter_elements().map(|c| c.as_char()).collect::<String>().as_str().lines().take(1).collect::<String>());

    let (rem, body) = match take_until("=").parse(input.clone()) {
        Ok((rem, body)) => {
            peek(recognize(stitch_signature)).parse(rem.clone())?;
            (rem, body)
        },
        nom::IResult::Err(nom::Err::Incomplete(_)) => {
            input.take_split(input.input_len()-1)
        },
        err => err?
    };
    
println!("\n<<<<\n{}\n<<<<\n", rem.iter_elements().map(|c| c.as_char()).collect::<String>().as_str().lines().take(1).collect::<String>());
    println!("End stitch body");
    Ok((rem, body))
}

fn knot_signature<I>(input: I) -> IResult<I, ast::Callable> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
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

fn stitch_signature<I>(input: I) -> IResult<I, ast::Callable> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{ 
    let (rem, (_, (name, _, parameters), _)) =
    (
        (space0, tag("="), space0),
        (identifier, space0, opt(parameter_list)),
        (space0, line_ending)
    ).parse(input)?;
    
    Ok((rem, ast::Callable{
        name: name.into(), 
        parameters: parameters.unwrap_or(vec![]), 
        ty: ast::Subprogram::Knot
    }))
}

fn identifier<I>(input: I) -> IResult<I, ast::Identifier> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    let (rem, name) = verify(
        //TODO: Add parsers for non-ASCII characters
        recognize(many0(alt((alphanumeric1, tag("_"))))),
        |n: &I| {let c = 
            n.iter_elements()
             .nth(0)
             .expect("Parsed identifier as empty string?!")
             .as_char();

            c.is_alpha() || c == '_'
        }).parse(input)?;

    use nom::AsChar;
    let name = name.iter_elements().map(|c| c.as_char()).collect();

    Ok((rem, name))
}

fn parameter_list<I>(input: I) -> IResult<I, Vec<ast::Parameter>> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
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
                Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: root_name, ..
                    }, 
                    root: Stitch{ body: root_body, ..},
                    ..
                }, 
            [
                Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: k1_name, ..
                    }, 
                    root: Stitch{ body: k1_body, ..},
                    ..
                }, 
                Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: k2_name, ..
                    }, 
                    root: Stitch{ body: k2_body, ..},
                    ..
                }, 
            ]) => {
                assert_eq!(root_name, "__root");
                assert_ne!(root_body.trim(), "", "Root body parse error");

                assert_eq!(k1_name, "K1");
                assert_ne!(k1_body.trim(), "K1 body parse error");

                assert_eq!(k2_name, "K2");
                assert_ne!(k2_body.trim(), "K2 body parse error");
            },
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
                Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: root_name, ..
                    }, 
                    root: Stitch{ body: root_body, ..},
                    ..
                }, 
            [
                Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: k1_name, ..
                    }, 
                    root: Stitch{ body: k1_body, ..},
                    ..
                }, 
                Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: k2_name, ..
                    }, 
                    root: Stitch{ body: k2_body, ..},
                    ..
                }, 
            ]) => {
                assert_eq!(root_name, "__root");
                assert_eq!(root_body.trim(), "", "Root body parse error");

                assert_eq!(k1_name, "K1");
                assert_ne!(k1_body.trim(), "K1 body parse error");

                assert_eq!(k2_name, "K2");
                assert_ne!(k2_body.trim(), "K2 body parse error");
            },
            _ => { panic!("Invalid parse.\nRemaining: \n{}\n---", unparsed); }
        };

        Ok(())
    }

    #[test]
    fn parse_knot_with_parameters() -> Result<(), Box<dyn std::error::Error>>    {
        let (unparsed, knots) = nom::multi::many1(knot).parse(include_str!("../tests/knots_with_parameters.ink"))?;
        match knots.as_slice() {
            [
                Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k1_parameters, ..}, ..}, 
                Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k2_parameters, ..}, ..}, 
                Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k3_parameters, ..}, ..}, 
                Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k4_parameters, ..}, ..}, 
                Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k5_parameters, ..}, ..}, 
                Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k6_parameters, ..}, ..}, 
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

    #[test]
    fn parse_identifier() -> Result<(), Box<dyn std::error::Error>> {
        let (res, id) = identifier("a;")?; assert_eq!((";", "a"), (res, id.as_str()));
        let (res, id) = identifier("var;")?; assert_eq!((";", "var"), (res, id.as_str()));
        let (res, id) = identifier("a_var;")?; assert_eq!((";", "a_var"), (res, id.as_str()));
        let (res, id) = identifier("_var;")?; assert_eq!((";", "_var"), (res, id.as_str()));
        let (res, id) = identifier("a_var_w_1_number;")?; assert_eq!((";", "a_var_w_1_number"), (res, id.as_str()));

        if let Ok((res, id)) = identifier("1var_w_num;") { panic!("Invalid parse! Should of returned error.\nresult: ('{}','{}')", res, id); }

        Ok(())
    }
}
