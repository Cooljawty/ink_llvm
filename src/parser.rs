use nom::IResult;
use nom::{
    Parser,
    multi::{many0},
    bytes::{take_till},
};

use crate::{ast};

trait Input: std::io::BufRead + nom::Input + Sized {}

pub fn parse<I: Input>(input: I) -> IResult<I, (ast::Subprogram, Vec<ast::Subprogram>)>
{
    let (remaining, (root, knots)) = (knot, many0(knot)).parse(input)?;
    let program = (root, knots);
    Ok((remaining, program))
}


fn knot<I: Input>(input: I) -> IResult<I, ast::Subprogram>
{
    let (remaining, knot_src) = take_till(|i| knot_signature(i).is_ok()).parse(input)?;
    let (remaining, (signature, body)) = (knot_signature, knot_body).parse(knot_src)?;

    Ok((remaining, ast::Subprogram::Knot))
}

fn knot_signature<I: Input>(input: I) -> IResult<I, ast::Subprogram> { todo!() }
fn knot_body<I: Input>(input: I) -> IResult<I, ast::Subprogram> { todo!() }

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
