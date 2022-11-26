extern crate proc_macro;
use absolution::{Ident, LitKind, Literal, Punct, PunctKind, TokenStream, TokenTree};
use quote::quote;

#[proc_macro]
pub fn fmt(tt: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match template_impl(tt) {
        Ok(r) => r,
        Err(msg) => quote! {
            ::std::compile_error!(#msg)
        }
        .into(),
    }
}

struct State<'a> {
    dst: &'a str,
    strb: &'a [u8],
    result: String,
    current_literal: Vec<u8>,
    trim_literal_start: bool,
    i: usize,
    paren_stack: Vec<ParenKind>,
    prefix: &'a str,
    suffix: &'a str,
}

#[derive(PartialEq, Eq, Debug)]
enum ParenKind {
    Paren,
    Bracket,
    Brace,
}

// Parse elements:
//   [stmts]  -> directly emit stmts as Rust code.
//   {expr}   -> evaluate `expr` and push it with ufmt
//   [[       -> [
//   {{       -> {
impl<'a> State<'a> {
    fn flush_literal(&mut self) {
        if !self.current_literal.is_empty() {
            let lit = std::str::from_utf8(&self.current_literal).unwrap();
            self.result.push_str(&format!(
                "{}.push_str({});\n",
                self.dst,
                proc_macro::Literal::string(lit).to_string()
            ));
            self.current_literal.clear();
        }
    }

    fn trim_literal_end(&mut self) {
        loop {
            if let Some(ref b) = self.current_literal.last() {
                if b.is_ascii_whitespace() {
                    self.current_literal.pop();
                    continue;
                }
            }
            break;
        }
    }

    fn apply_trimming<'s>(&mut self, mut rs_code: &'s str) -> &'s str{
        self.trim_literal_start = false;
        if !rs_code.is_empty() {
            if rs_code.as_bytes()[0] == b'-' {
                self.trim_literal_end();
                rs_code = &rs_code[1..];
            }
            if !rs_code.is_empty() && rs_code.as_bytes()[rs_code.len() - 1] == b'-' {
                rs_code = &rs_code[..rs_code.len() - 1];
                self.trim_literal_start = true;
            }
        }
        rs_code
    }

    fn parse_stmts(&mut self) -> Result<(), String> {
        let rs_code = self.scan_parens(ParenKind::Bracket, true)?;
        let rs_code = self.apply_trimming(rs_code);
        if !rs_code.is_empty() {
            self.flush_literal();
        }
        self.result.push_str(rs_code);
        Ok(())
    }

    fn parse_expr(&mut self) -> Result<(), String> {
        let rs_code = self.scan_parens(ParenKind::Brace, false)?;
        let rs_code = self.apply_trimming(rs_code);
        self.flush_literal();
        self.result.push_str(&format!(
            "::utemplate::TDisplay::tdisplay_to({rs_code}, {dst});\n",
            dst = self.dst
        ));
        Ok(())
    }

    fn scan_literal(&mut self, ender: u8) -> Result<(), String> {
        // Search for the first unescaped occurence of ender.
        loop {
            let b = *self.strb.get(self.i).ok_or("unterminated literal")?;
            self.i += 1;
            if b == b'\'' {
                self.i += 1;
                continue;
            } else if b == ender {
                return Ok(());
            }
        }
    }

    fn scan_parens(
        &mut self,
        start: ParenKind,
        allow_unbalanced_brace: bool,
    ) -> Result<&'a str, String> {
        let scan_start = self.i;
        self.paren_stack.clear();
        self.paren_stack.push(start);
        loop {
            if self.paren_stack.is_empty() {
                return Ok(std::str::from_utf8(&self.strb[scan_start..self.i - 1]).unwrap());
            }
            let b = *self.strb.get(self.i).ok_or("unterminated paren")?;
            self.i += 1;
            if b == b'\'' {
                self.scan_literal(b'\'')?;
            } else if b == b'"' {
                self.scan_literal(b'"')?;
            } else if b == b'[' {
                self.paren_stack.push(ParenKind::Bracket);
            } else if b == b'{' {
                self.paren_stack.push(ParenKind::Brace);
            } else if b == b'(' {
                self.paren_stack.push(ParenKind::Paren);
            } else if b == b'}' {
                if allow_unbalanced_brace && self.paren_stack.len() == 1 {
                } else if let Some(ParenKind::Brace) = self.paren_stack.pop() {
                } else {
                    return Err("unbalanced paren".to_string());
                }
            } else if b == b')' {
                if let Some(ParenKind::Paren) = self.paren_stack.pop() {
                } else {
                    return Err("unbalanced paren".to_string());
                }
            } else if b == b']' {
                let p = self.paren_stack.pop();
                if let Some(ParenKind::Bracket) = p {
                } else {
                    // when allow_unbalanced_brace, we allow p and everything
                    // in self.paren_stack to be Brace, without errors. Otherwise,
                    // we error.
                    if allow_unbalanced_brace
                        && p == Some(ParenKind::Brace)
                        && self.paren_stack.first() == Some(&ParenKind::Bracket)
                        && self.paren_stack[1..].iter().all(|p| *p == ParenKind::Brace)
                    {
                        self.paren_stack.clear();
                    } else {
                        return Err("unbalanced paren".to_string());
                    }
                }
            }
        }
    }

    fn parse_root(mut self) -> Result<String, String> {
        self.result.push_str("{\n");
        self.result.push_str(&self.prefix);
        while self.i < self.strb.len() {
            let b = self.strb[self.i];
            self.i += 1;
            if b == b'[' {
                let b = *self.strb.get(self.i).ok_or("unexpected end of template")?;
                // Peek ahead for "{{"
                if b == b'[' {
                    self.i += 1;
                    self.current_literal.push(b'[');
                    continue;
                } else {
                    self.parse_stmts()?;
                }
            } else if b == b'{' {
                let b = *self.strb.get(self.i).ok_or("unexpected end of template")?;
                // Peek ahead for "{{"
                if b == b'{' {
                    self.i += 1;
                    self.current_literal.push(b'{');
                    continue;
                } else {
                    self.parse_expr()?;
                }
            } else {
                if self.trim_literal_start {
                    if b.is_ascii_whitespace() {
                        // Don't push it onto the literal
                        continue;
                    }
                    self.trim_literal_start = false;
                }
                self.current_literal.push(b);
            }
        }
        self.flush_literal();
        self.result.push_str(&self.suffix);
        self.result.push_str("}\n");
        Ok(self.result)
    }
}

fn template_impl(tt: proc_macro::TokenStream) -> Result<proc_macro::TokenStream, String> {
    let stream: TokenStream = tt.into();
    let (prefix, dst, str, suffix) = match &stream.tokens[..] {
        // "..."   (create a new string, not nameable)
        // dst = "..."   (create a new string, nameable as `dst`)
        // [
        //     TokenTree::Literal(Literal { kind: LitKind::Str(str), .. }),
        // ],
        // "..."  (append to *dst, which has type `String`)
        [
            TokenTree::Literal(Literal { kind: LitKind::Str(str), .. }),
        ] => ("let mut __utemplate_macro_dst = ::std::string::String::new();\n", "(&mut __utemplate_macro_dst)".to_string(), str, "__utemplate_macro_dst"),
        // *dst += "..."  (append to *dst, which has type `String`)
        [
            TokenTree::Punct(Punct { kind: PunctKind::Star, .. }),
            TokenTree::Ident(Ident { ident: dst, .. }),
            TokenTree::Punct(Punct { kind: PunctKind::Plus, .. }),
            TokenTree::Punct(Punct { kind: PunctKind::Eq, .. }),
            TokenTree::Literal(Literal { kind: LitKind::Str(str), .. }),
        ] => ("", dst.clone(), str, ""),
        // dst += "..."  (append to dst, which has type `String`)
        [
            TokenTree::Ident(Ident { ident: dst, .. }),
            TokenTree::Punct(Punct { kind: PunctKind::Plus, .. }),
            TokenTree::Punct(Punct { kind: PunctKind::Eq, .. }),
            TokenTree::Literal(Literal { kind: LitKind::Str(str), .. }),
        ] => ("", format!("(&mut {dst})"), str, ""),
        _ => {
            let msg = format!("bad template format, got:\n{:#?}\n", &stream.tokens[..]);
            return Err(msg);
        },
    };
    let state = State {
        result: String::new(),
        current_literal: Vec::new(),
        trim_literal_start: false,
        i: 0,
        dst: &dst[..],
        strb: str.as_bytes(),
        paren_stack: Vec::new(),
        prefix,
        suffix,
    };

    state
        .parse_root()?
        .parse::<proc_macro::TokenStream>()
        .map_err(|e| e.to_string())
}
