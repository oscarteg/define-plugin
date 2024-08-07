use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use swc_core::ecma::{
    ast::{Expr, Ident, Lit, Program, Str},
    atoms::JsWord,
    transforms::testing::test_inline,
    visit::{as_folder, FoldWith, VisitMut},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    replacements: HashMap<String, String>,
}

pub struct EnvVarReplacer {
    replacements: HashMap<String, String>,
}

impl EnvVarReplacer {
    fn new(config: Config) -> Self {
        Self {
            replacements: config.replacements,
        }
    }
}

impl VisitMut for EnvVarReplacer {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Expr::Ident(ident) = expr {
            if let Some(env_var_name) = self.replacements.get(&ident.sym.to_string()) {
                if let Ok(env_value) = env::var(env_var_name) {
                    *expr = Expr::Lit(Lit::Str(Str {
                        span: ident.span,
                        value: JsWord::from(env_value),
                        raw: None,
                    }));
                }
            }
        }
    }
}

#[plugin_transform]
pub fn env_var_replacer(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config: Config = serde_json::from_str(&metadata.get_transform_plugin_config().unwrap())
        .expect("Invalid plugin configuration");
    program.fold_with(&mut as_folder(EnvVarReplacer::new(config)))
}