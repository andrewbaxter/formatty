use std::{
    fmt::Write,
};
use syn::{
    Expr,
    FieldPat,
    Pat,
};
use crate::{
    new_sg,
    new_sg_lit,
    sg_general::{
        append_binary,
        append_inline_list_raw,
        new_sg_outer_attrs,
        new_sg_binary,
        new_sg_comma_bracketed_list,
        new_sg_comma_bracketed_list_ext,
        new_sg_macro,
        append_comments,
    },
    sg_type::{
        build_extended_path,
        build_ref,
    },
    Alignment,
    Formattable,
    MakeSegsState,
    TrivialLineColMath,
    SplitGroupIdx,
};

impl Formattable for &Pat {
    fn make_segs(&self, out: &mut MakeSegsState, base_indent: &Alignment) -> SplitGroupIdx {
        match self {
            Pat::Box(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    let mut node = new_sg(out);
                    node.seg(out, "box ");
                    node.child(x.pat.as_ref().make_segs(out, base_indent));
                    node.build(out)
                },
            ),
            Pat::Ident(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    let mut prefix = String::new();
                    let mut start = None;
                    if let Some(y) = x.by_ref {
                        prefix.write_str("ref ").unwrap();
                        start = Some(y.span.start());
                    }
                    if let Some(y) = x.mutability {
                        prefix.write_str("mut ").unwrap();
                        if start.is_none() {
                            start = Some(y.span.start());
                        }
                    }
                    prefix.write_str(&x.ident.to_string()).unwrap();
                    if start.is_none() {
                        start = Some(x.ident.span().start());
                    }
                    if let Some(at) = &x.subpat {
                        new_sg_binary(out, base_indent, |out: &mut MakeSegsState, base_indent: &Alignment| {
                            new_sg_lit(out, start.map(|s| (base_indent, s)), &prefix)
                        }, at.0.span.start(), " @", &*at.1)
                    } else {
                        new_sg_lit(out, start.map(|s| (base_indent, s)), prefix)
                    }
                },
            ),
            Pat::Lit(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    x.expr.as_ref().make_segs(out, base_indent)
                },
            ),
            Pat::Macro(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    new_sg_macro(out, base_indent, &x.mac, false)
                },
            ),
            Pat::Or(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    let mut sg = new_sg(out);
                    if let Some(t) = &x.leading_vert {
                        append_comments(out, base_indent, &mut sg, t.span.start());
                        sg.seg(out, "| ");
                    }
                    append_inline_list_raw(out, base_indent, &mut sg, " |", false, &x.cases);
                    sg.build(out)
                },
            ),
            Pat::Path(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    build_extended_path(out, base_indent, &x.qself, &x.path)
                },
            ),
            Pat::Range(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    let (tok_loc, tok) = match x.limits {
                        syn::RangeLimits::HalfOpen(x) => (x.spans[0].start(), ".."),
                        syn::RangeLimits::Closed(x) => (x.spans[0].start(), "..="),
                    };
                    new_sg_binary(out, base_indent, x.lo.as_ref(), tok_loc, tok, x.hi.as_ref())
                },
            ),
            Pat::Reference(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    build_ref(out, base_indent, x.and_token.span.start(), x.mutability.is_some(), x.pat.as_ref())
                },
            ),
            Pat::Rest(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    new_sg_lit(out, Some((base_indent, x.dot2_token.spans[0].start())), "..")
                },
            ),
            Pat::Slice(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    new_sg_comma_bracketed_list(
                        out,
                        base_indent,
                        None::<Expr>,
                        x.bracket_token.span.start(),
                        "[",
                        &x.elems,
                        x.bracket_token.span.end().prev(),
                        "]",
                    )
                },
            ),
            Pat::Struct(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    if let Some(d) = x.dot2_token {
                        new_sg_comma_bracketed_list_ext(
                            out,
                            base_indent,
                            Some(&x.path),
                            x.brace_token.span.start(),
                            " {",
                            &x.fields,
                            |out: &mut MakeSegsState, base_indent: &Alignment| {
                                new_sg_lit(out, Some((base_indent, d.spans[0].start())), "..")
                            },
                            "}",
                        )
                    } else {
                        new_sg_comma_bracketed_list(
                            out,
                            base_indent,
                            Some(&x.path),
                            x.brace_token.span.start(),
                            " {",
                            &x.fields,
                            x.brace_token.span.end().prev(),
                            "}",
                        )
                    }
                },
            ),
            Pat::Tuple(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    new_sg_comma_bracketed_list(
                        out,
                        base_indent,
                        None::<Expr>,
                        x.paren_token.span.start(),
                        "(",
                        &x.elems,
                        x.paren_token.span.end().prev(),
                        ")",
                    )
                },
            ),
            Pat::TupleStruct(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    new_sg_comma_bracketed_list(
                        out,
                        base_indent,
                        Some(&x.path),
                        x.pat.paren_token.span.start(),
                        "(",
                        &x.pat.elems,
                        x.pat.paren_token.span.end().prev(),
                        ")",
                    )
                },
            ),
            Pat::Type(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, base_indent: &Alignment| {
                    new_sg_binary(out, base_indent, x.pat.as_ref(), x.colon_token.span.start(), ":", x.ty.as_ref())
                },
            ),
            Pat::Verbatim(x) => new_sg_lit(out, None, x),
            Pat::Wild(x) => new_sg_outer_attrs(
                out,
                base_indent,
                &x.attrs,
                |out: &mut MakeSegsState, _base_indent: &Alignment| {
                    new_sg_lit(out, Some((base_indent, x.underscore_token.span.start())), "_")
                },
            ),
            _ => unreachable!(),
        }
    }
}

impl Formattable for Pat {
    fn make_segs(&self, out: &mut MakeSegsState, base_indent: &Alignment) -> SplitGroupIdx {
        (&self).make_segs(out, base_indent)
    }
}

impl Formattable for FieldPat {
    fn make_segs(&self, out: &mut MakeSegsState, base_indent: &Alignment) -> SplitGroupIdx {
        new_sg_outer_attrs(out, base_indent, &self.attrs, |out: &mut MakeSegsState, base_indent: &Alignment| {
            let mut sg = new_sg(out);
            match &self.member {
                syn::Member::Named(x) => sg.seg(out, x),
                syn::Member::Unnamed(x) => sg.seg(out, x.index),
            };
            append_binary(out, base_indent, &mut sg, ":", self.pat.as_ref());
            sg.build(out)
        })
    }
}
