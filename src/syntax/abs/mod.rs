use crate::check::monad::error::TCE;
use crate::check::monad::TCM;
use crate::syntax::common::{DtKind, Level, ParamKind, SyntaxInfo, DBI};
use crate::syntax::env::NamedEnv_;
use crate::syntax::surf::ast::{Decl, DeclKind, Expr};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum Abstract {
    Type(SyntaxInfo, Level),
    /// Bottom type
    Bot(SyntaxInfo),
    /// Global variable
    Var(SyntaxInfo, DBI),
    /// Local variable
    Local(SyntaxInfo, DBI),
    /// Construct call
    Cons(SyntaxInfo, Box<Self>),
    /// Construct call
    ConsType(SyntaxInfo, Box<Self>),
    /// Apply or Pipeline in surface
    App(Box<Self>, Box<Self>),
    /// Dependent Type type
    Dt(DtKind, Box<Abstract>, Box<Abstract>),
    Pair(SyntaxInfo, Box<Self>, Box<Self>),
    Fst(SyntaxInfo, Box<Self>),
    Snd(SyntaxInfo, Box<Self>),
}

/// type signature and value in abstract syntax
#[derive(Debug, Clone)]
pub enum AbstractDecl {
    JustSign(Abstract),
    JustImpl(Abstract),
    Decl(Abstract, Abstract),
}

pub type AbstractGlobalEnv = VecDbiEnv_<AbstractDecl>;

pub type VecDbiEnv_<T> = BTreeMap<DBI, T>;

/// todo: replace to proper location
pub fn trans(decls: Vec<Decl>) -> TCM<AbstractGlobalEnv> {
    decls
        .iter()
        .fold(
            Ok(Default::default()),
            |result: TCM<(AbstractGlobalEnv, NamedEnv_<DBI>)>, decl| {
                let name = decl.name.clone();
                let (mut result, mut name_map) = result?;

                let abs = trans_expr(&decl.body, &result, &name_map)?;

                let dbi = name_map
                    .entry(name.info.text)
                    .or_insert(result.len())
                    .clone();
                let abs_decl = result.get(&dbi);
                result.insert(
                    dbi,
                    match (decl.kind, abs_decl.clone()) {
                        (DeclKind::Sign, None)
                        | (DeclKind::Sign, Some(AbstractDecl::JustSign(_))) => {
                            AbstractDecl::JustSign(abs)
                        }
                        (DeclKind::Impl, None)
                        | (DeclKind::Impl, Some(AbstractDecl::JustImpl(_))) => {
                            AbstractDecl::JustImpl(abs)
                        }
                        (DeclKind::Sign, Some(AbstractDecl::JustImpl(impl_abs)))
                        | (DeclKind::Sign, Some(AbstractDecl::Decl(_, impl_abs))) => {
                            AbstractDecl::Decl(abs, impl_abs.clone())
                        }
                        (DeclKind::Impl, Some(AbstractDecl::JustSign(sign_abs)))
                        | (DeclKind::Impl, Some(AbstractDecl::Decl(sign_abs, _))) => {
                            AbstractDecl::Decl(sign_abs.clone(), abs)
                        }
                    },
                );
                Ok((result, name_map))
            },
        )
        .map(|(result, _)| result)
}

pub fn trans_expr(expr: &Expr, env: &AbstractGlobalEnv, map: &NamedEnv_<DBI>) -> TCM<Abstract> {
    trans_expr_inner(expr, env, map, &BTreeMap::new(), &BTreeMap::new())
}

pub fn trans_expr_inner(
    expr: &Expr,
    env: &AbstractGlobalEnv,
    global_map: &NamedEnv_<DBI>,
    local_env: &AbstractGlobalEnv,
    local_map: &NamedEnv_<DBI>,
) -> TCM<Abstract> {
    match expr {
        Expr::Type(syntax, level) => Ok(Abstract::Type(syntax.clone(), *level)),
        Expr::Var(ident) => {
            let name = ident.info.text.clone();
            if local_map.contains_key(&name) {
                Ok(Abstract::Local(ident.info.clone(), local_map[&name]))
            } else if global_map.contains_key(&name) {
                Ok(Abstract::Var(ident.info.clone(), global_map[&name]))
            } else {
                Err(TCE::LookUpFailed(name))
            }
        }
        Expr::App(app_vec) => app_vec
            .iter()
            .try_fold(
                None,
                |result: Option<Abstract>, each_expr| -> TCM<Option<Abstract>> {
                    let abs = trans_expr_inner(each_expr, env, global_map, local_env, local_map)?;
                    Ok(match result {
                        // First item in vec
                        None => Some(abs),
                        // Second or other, reduce to Right
                        Some(left_abs) => Some(Abstract::App(Box::new(left_abs), Box::new(abs))),
                    })
                },
            )
            .map(|option_abs| option_abs.unwrap()),
        Expr::Pipe(pipe_vec) => {
            let mut app_vec = pipe_vec.clone();
            app_vec.reverse();
            trans_expr_inner(&Expr::App(app_vec), env, global_map, local_env, local_map)
        }
        Expr::Meta(_) => unimplemented!(),
        Expr::Cons(_) => unimplemented!(),
        Expr::ConsType(_) => unimplemented!(),
        Expr::Bot(ident) => Ok(Abstract::Bot(ident.info.clone())),
        Expr::Sum(_) => unimplemented!(),
        Expr::Pi(params, result) => {
            let mut pi_env = local_env.clone();
            let mut pi_map = local_map.clone();
            let mut pi_vec: Vec<Abstract> = params.iter().try_fold(
                Vec::new(),
                |mut pi_vec: Vec<Abstract>, param| -> TCM<Vec<Abstract>> {
                    // todo: handle implicit parameter
                    assert_eq!(param.kind, ParamKind::Explicit);
                    let param_ty =
                        trans_expr_inner(&param.ty.clone(), env, global_map, &pi_env, &pi_map)?;
                    for name in param.names.clone() {
                        let param_name = name.info.text;
                        let param_dbi: DBI = local_env.len();
                        pi_env.insert(param_dbi, AbstractDecl::JustSign(param_ty.clone()));
                        pi_map.insert(param_name, param_dbi);
                    }
                    pi_vec.insert(pi_vec.len(), param_ty);
                    Ok(pi_vec)
                },
            )?;

            // fold from right
            pi_vec.reverse();
            Ok(pi_vec.iter().fold(
                trans_expr_inner(result, env, global_map, &pi_env, &pi_map)?,
                |pi_abs, param| Abstract::Dt(DtKind::Pi, Box::new(param.clone()), Box::new(pi_abs)),
            ))
        }
    }
}
