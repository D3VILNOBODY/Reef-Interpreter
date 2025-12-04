pub mod lex;
pub mod parse;
pub mod syntax;

/// A small test suite to quickly test the functions of the lexer and
/// parser. `test_lexer_and_parser` is a util function that creates a
/// lexer with `source_code`, and then scans the code, then makes a
/// parser and parses the tokens.
#[cfg(test)]
mod tests {
    use super::*;

    fn test_lexer_and_parser(source_code: &str) {
        let mut l = lex::Scanner::new(source_code);
        l.scan();

        let mut p = parse::Parser::new(l.tokens);
        let res = p.parse();

        if res.is_err() {
            println!("=-=- Source Code -=-=");
            println!("{}", source_code);
            println!("=-=-=-=-=-=-=-=-=-=-=");
            panic!("{:?}", res.unwrap_err());
        }
    }

    #[test]
    fn passes() {
        test_lexer_and_parser("var money = 100;");
    }

    #[test]
    fn should_fail() {
        test_lexer_and_parser("var = 10");
    }
}
