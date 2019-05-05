use crate::syntax::abs::Abs;
use crate::syntax::common::{DtKind::*, ParamKind::*};
use crate::syntax::core::{RedEx, Term};

use super::monad::{TermTCM, TCE, TCM, TCS};

/// $$
/// \newcommand\U{\textsf{Type}}
/// \frac{i < j}{\Gamma \vdash \U\_i : \U\_j \rightsquigarrow \U\_i}
/// \quad
/// \frac{}{\Gamma \vdash\bot:\U\_i \rightsquigarrow \bot\_i}
/// $$
/// $$
/// \frac{\Gamma \vdash a:A \quad \Gamma,a:A \vdash b:B(a)}
///      {\Gamma \vdash (a,b):\Sigma (a:A).B(a) \rightsquigarrow (a,b)}
/// $$
/// Abstract Term -> Core Term.
pub fn check(tcs: TCS, expr: Abs, expected_type: Term) -> TermTCM {
    match (expr, expected_type) {
        (Abs::Type(info, lower), Term::Type(upper)) => {
            if upper > lower {
                Ok((tcs, Term::Type(lower).into_info(info)))
            } else {
                Err(TCE::LevelMismatch(info, lower + 1, upper))
            }
        }
        (Abs::Pair(info, fst, snd), Term::Dt(Explicit, Sigma, snd_ty)) => {
            let (tcs, fst_term) = check(tcs, *fst, *snd_ty.param_type)?;
            let snd_ty = snd_ty.body.reduce(fst_term.ast.clone());
            let (tcs, snd_term) = check(tcs, *snd, snd_ty)?;
            Ok((tcs, Term::pair(fst_term.ast, snd_term.ast).into_info(info)))
        }
        (Abs::Bot(info), Term::Type(level)) => Ok((tcs, Term::Bot(level - 1).into_info(info))),
        _ => unimplemented!(),
    }
}

/// Check if an expression is a valid type expression
pub fn check_type(_tcs: TCS, _expr: Abs) -> TermTCM {
    unimplemented!()
}

/// infer type of value
pub fn check_infer(tcs: TCS, value: Abs) -> TermTCM {
    use crate::syntax::abs::Abs::*;
    match value {
        Type(info, level) => Ok((tcs, Term::Type(level + 1).into_info(info))),
        Local(info, dbi) => {
            let ty = tcs.local_gamma[dbi].r#type.clone().into_info(info);
            Ok((tcs, ty))
        }
        Var(info, dbi) => {
            let ty = tcs.gamma[dbi].r#type.clone().into_info(info);
            Ok((tcs, ty))
        }
        Pair(info, fst, snd) => {
            let (tcs0, fst_ty) = check_infer(tcs, *fst)?;
            let (tcs1, snd_ty) = check_infer(tcs0, *snd)?;
            Ok((tcs1, Term::pair(fst_ty.ast, snd_ty.ast).into_info(info)))
        }
        Fst(info, pair) => {
            let (new_tcs, pair_ty) = check_infer(tcs, *pair)?;
            match pair_ty.ast {
                Term::Dt(Explicit, Sigma, closure) => {
                    Ok((new_tcs, closure.param_type.into_info(info)))
                }
                ast => Err(TCE::NotSigma(pair_ty.info, ast)),
            }
        }
        // ConsType(info) => Ok((
        //     tcs,
        //     Term::Lam(Closure::new(Term::gen(0), ClosureBody::new(sum))),
        // )),
        _ => unimplemented!(),
    }
}

/// check if type1 is subtype of type2
pub fn check_subtype(tcs: TCS, subtype: &Term, supertype: &Term) -> TCM {
    use crate::syntax::core::Term::*;
    match (subtype, supertype) {
        (Type(sub_level), Type(super_level)) if sub_level <= super_level => Ok(tcs),
        _ => unimplemented!(),
    }
}
