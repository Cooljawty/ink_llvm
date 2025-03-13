use nom::IResult;

/* Debug printer

println!("\n>>>>\n{}\n>>>>\n", input.iter_elements().map(|c| c.as_char()).collect::<String>().as_str().lines().next().unwrap());
println!("\n<<<<\n{}\n<<<<\n", rem.iter_elements().map(|c| c.as_char()).collect::<String>().as_str().lines().next().unwrap());
*/

#[allow(unused_imports)]
use nom::{
    AsChar,
    Parser,
    multi::{many0, many_till, fold_many0,separated_list0,many1},
    bytes::{tag, is_a, take_until,take_till},
    character::{
        anychar,one_of,
        complete::{
            space0,
            alpha1, alphanumeric1,
            line_ending, not_line_ending,
        },
    },
    combinator::{value, opt, eof, not, peek, recognize,success,all_consuming,flat_map,verify, complete},
    branch::{alt,},
};

use crate::{ast};

//type Program = (Vec<ast::Knot>)
pub fn parse<I>(input: I) -> IResult<I, (ast::Knot<I>, Vec<ast::Knot<I>>) >
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    println!("== __root ==");
    let (remaining, (root_root_stitch, root_stitches)) = knot_body.parse(input)?;
    println!("= =");
    let (remaining, (knots, _)) = (many0(knot), alt((line_ending, eof))).parse(remaining)?;

    let root_signature = ast::Callable{ ty: ast::Subprogram::Knot, name: "__root".into(), parameters: vec![], };

    let program = (
        ast::Knot {
            signature: root_signature, 
            root:      root_root_stitch,
            body:      root_stitches
        },
        knots
    );

    //
    //println!("\n== {} ==",   program.0.signature.name);
    //println!("{}\n========", program.0.root.body.iter_elements().map(|c| c.as_char()).collect::<String>());
    Ok((remaining, program))
}


fn knot<I>(input: I) -> IResult<I, ast::Knot<I>>
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{   
    let (rem, signature) = knot_signature.parse(input)?;
    println!("\n== {} ==", signature.name);
    let (rem, (root_stitch, stitches)) = knot_body.parse(rem)?;
    println!("== ==");

    Ok ((rem, ast::Knot {
            signature: signature, 
            root:      root_stitch,
            body:      stitches
    }))
}

fn knot_body<I>(input: I) -> IResult<I, (ast::Stitch<I>, Vec<ast::Stitch<I>>)> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{
    /*
    let (rem, body) = recognize(many0((
            peek(not(recognize(knot_signature))),
            many0(not_line_ending),
            alt((line_ending, eof)),
    ))).parse(input)?;
    */
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
    
    let (_, (body, stitches)) = (stitch_body, many0(complete(stitch))).parse(body)?;

    let root_stitch = ast::Stitch {
        signature: ast::Callable{ 
            ty: ast::Subprogram::Stitch, 
            name: "__root".into(), 
            parameters: vec![], 
        }, 
        body,
    };
    Ok((rem, (root_stitch, stitches)))
}

fn stitch<I>(input: I) -> IResult<I, ast::Stitch<I>>
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{   
    let (rem, signature) = stitch_signature.parse(input)?;
    println!("\n= {}", signature.name);

    println!("input: {:?}\n",
        rem.iter_elements().map(|c| c.as_char()).collect::<String>(),
    );
    let (rem, body) = stitch_body.parse(rem)?;
    //
    println!("= =");
    Ok((rem, ast::Stitch{signature, body}))
}

fn stitch_body<I>(input: I) -> IResult<I, I> 
where
	for<'p> I: nom::Input + nom::Offset + nom::Compare<&'p str> + nom::FindSubstring<&'p str>,
    <I as nom::Input>::Item: nom::AsChar,
    for<'p> &'p str: nom::FindToken<<I as nom::Input>::Item>,
{ 
    
    //println!(">>>\n{:?}\n>>>\n",
    //    input.iter_elements().map(|c| c.as_char()).collect::<String>(),
    //);

    if input.input_len() == 0 { return Ok((input.clone(), input.take_from(0))) }
    
    let rem = input.clone();
    let mut body_size = 0;
    let (rem, body) = loop {
        let (rem, line) = peek(recognize((not_line_ending, opt(line_ending)))).parse(rem.take_from(body_size))?;
        //println!("line: {:?}", line.iter_elements().map(|c| c.as_char()).collect::<String>());

        let res: IResult<I, ()> = peek(not((
            space0, 
            alt(
                (eof, tag("="))
            )
        ))).parse(line.clone());
        if res.is_ok() { 
            body_size += line.input_len(); 
        }
        else { 
            break (rem, input.take(body_size)) 
        }
    };
    println!("body: {:?}\nrem: {:?}\n", 
        body.iter_elements().map(|c| c.as_char()).collect::<String>(),
        rem.iter_elements().map(|c| c.as_char()).collect::<String>()
    );
    /*
    
    println!("---\n{}\n>>>\n{}\n<<<\n{}\n---\n",
        input.iter_elements().map(|c| c.as_char()).collect::<String>(),
        rem.iter_elements().map(|c| c.as_char()).collect::<String>(),
        body.iter_elements().map(|c| c.as_char()).collect::<String>(),
    );
    */

    /*
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
    */
    
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
                ast::Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: root_name, ..
                    }, 
                    root: ast::Stitch{ body: root_body, ..},
                    body: root_stitches,
                    ..
                }, 
            [
                ast::Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: k1_name, ..
                    }, 
                    root: ast::Stitch{ body: k1_body, ..},
                    body: k1_stitches,
                    ..
                }, 
                ast::Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: k2_name, ..
                    }, 
                    root: ast::Stitch{ body: k2_body, ..},
                    body: k2_stitches,
                    ..
                }, 
            ]) => {
                assert_eq!(root_name, "__root");
                assert_ne!(root_body.trim(), "", "Root body parse error");
                assert_ne!(root_stitches[0].body.trim(), "", "Root stitch body parse error");

                assert_eq!(k1_name, "K1");
                assert_ne!(k1_body.trim(), "", "K1 body parse error");

                assert_eq!(k1_stitches[0].signature.name, "K1_1", "K1_1 body parse error");
                assert_ne!(k1_stitches[0].body.trim(), "", "K1_1 body parse error");

                assert_eq!(k2_name, "K2");
                assert_eq!(k2_body.trim(), "", "K1 body parse error");

                assert_eq!(k2_stitches[0].signature.name, "K2_1", "K2_1 body parse error");
                assert_ne!(k2_stitches[0].body.trim(), "", "K2_1 body parse error");
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
                ast::Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: root_name, ..
                    }, 
                    root: ast::Stitch{ body: root_body, ..},
                    body: root_stitches,
                    ..
                }, 
            [
                ast::Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: k1_name, ..
                    }, 
                    root: ast::Stitch{ body: k1_body, ..},
                    body: k1_stitches,
                    ..
                }, 
                ast::Knot{
                    signature: ast::Callable {
                        ty: ast::Subprogram::Knot, 
                        name: k2_name, ..
                    }, 
                    root: ast::Stitch{ body: k2_body, ..},
                    body: k2_stitches,
                    ..
                }, 
            ]) => {
                assert_eq!(root_name, "__root");
                assert_eq!(root_body.trim(), "", "Root body parse error");
                assert!(root_stitches.len() == 0, "Incorectly parsed root knot");

                assert_eq!(k1_name, "K1");
                assert_ne!(k1_body.trim(), "", "K1 body parse error");

                assert_eq!(k1_stitches[0].signature.name, "K1_1", "K1_1 body parse error");
                assert_ne!(k1_stitches[0].body.trim(), "", "K1_1 body parse error");

                assert_eq!(k2_name, "K2");
                assert_eq!(k2_body.trim(), "", "K1 body parse error");

                assert_eq!(k2_stitches[0].signature.name, "K2_1", "K2_1 body parse error");
                assert_ne!(k2_stitches[0].body.trim(), "", "K2_1 body parse error");
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
                ast::Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k1_parameters, ..}, ..}, 
                ast::Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k2_parameters, ..}, ..}, 
                ast::Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k3_parameters, ..}, ..}, 
                ast::Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k4_parameters, ..}, ..}, 
                ast::Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k5_parameters, ..}, ..}, 
                ast::Knot{ signature: ast::Callable { ty: ast::Subprogram::Knot, parameters: k6_parameters, ..}, ..}, 
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
