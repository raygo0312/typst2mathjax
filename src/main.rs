use itertools::Itertools;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::OnceLock;
use typst::syntax::{ast::AstNode, *};
use walkdir::WalkDir;
use wasm_bindgen::prelude::*;

static DICT: OnceLock<Value> = OnceLock::new();
const PAGES_DIR: &str = "pages";
const STATIC_DIR: &str = "static";
const DIST_DIR: &str = "dist";
const DICT_JSON: &str = include_str!("dictionary.json");

fn main() -> io::Result<()> {
  // 辞書をロード
  DICT.get_or_init(load_dict);

  fs::create_dir_all(DIST_DIR)?;
  copy_public_assets()?;

  for entry in WalkDir::new(PAGES_DIR)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| {
      e.path().is_file()
        && e
          .path()
          .extension()
          .map(|ext| ext == "html")
          .unwrap_or(false)
    })
  {
    let rel_path = entry.path().strip_prefix(PAGES_DIR).unwrap();
    let dist_path = Path::new(DIST_DIR).join(rel_path);
    if let Some(parent) = dist_path.parent() {
      fs::create_dir_all(parent)?;
    }

    let input = fs::read_to_string(entry.path())?;
    let output = transform_from_typst(&input);
    fs::write(&dist_path, output)?;
    println!("✅ Processed: {}", entry.path().display());
  }

  Ok(())
}

pub fn load_dict() -> Value {
  match serde_json::from_str(DICT_JSON) {
    Ok(val) => val,
    Err(e) => {
      eprintln!("JSONパースエラー: {}", e);
      Value::Object(serde_json::Map::new())
    }
  }
}

// Typst変換関数
#[wasm_bindgen]
pub fn transform_from_typst(input: &str) -> String {
  let re = Regex::new(r"(?s)\$(.+?)\$").unwrap();

  re.replace_all(input, |caps: &regex::Captures| {
    let typst_code = &caps[0];

    // Typst構文としてパース
    let parsed = parse(typst_code);
    let markup = match ast::Markup::from_untyped(&parsed) {
      Some(m) => m,
      None => return format!("<!-- parse error: {} -->", typst_code),
    };

    if let Some(expr) = markup.exprs().next() {
      if let ast::Expr::Equation(eq) = expr {
        let math = eq.body();
        let mathjax_code = analysis_math(math);

        if eq.block() {
          if typst_code.contains('\n') {
            return format!("\\begin{{align*}}{}\\end{{align*}}", mathjax_code);
          } else {
            return format!("\\[{}\\]", mathjax_code);
          }
        } else {
          return format!("\\({}\\)", mathjax_code);
        }
      }
    }
    format!("<!-- no expr: {:?} -->", markup)
  })
  .to_string()
}

fn typst_to_mathjax(math: ast::Expr) -> String {
  match math {
    ast::Expr::Math(math) => analysis_math(math),
    ast::Expr::FuncCall(text) => analysis_func_call(text),
    ast::Expr::Str(text) => format!("\\rm{{{}}}", text.get()),
    ast::Expr::MathText(text) => analysis_math_text(text),
    ast::Expr::FieldAccess(text) => analysis_field_access(text),
    ast::Expr::MathIdent(text) => analysis_math_indent(text),
    ast::Expr::MathShorthand(text) => analysis_math_shorthand(text),
    ast::Expr::MathAttach(text) => analysis_math_attach(text),
    ast::Expr::MathDelimited(text) => analysis_math_delimited(text),
    ast::Expr::MathFrac(text) => analysis_math_frac(text),
    ast::Expr::MathAlignPoint(_) => "&".to_string(),
    ast::Expr::Space(_) => " ".to_string(),
    ast::Expr::Linebreak(_) => "\\\\".to_string(),
    _ => format!("<!-- parse error: {:?} -->", math),
  }
}

fn analysis_math(math: ast::Math) -> String {
  math.exprs().map(typst_to_mathjax).collect()
}

fn analysis_func_call(text: ast::FuncCall) -> String {
  let dict = DICT.get().unwrap();

  let ast::Expr::MathIdent(callee) = text.callee() else {
    return format!("<!-- unsupported function call: {:?} -->", text.callee());
  };

  let Some(Value::String(s)) = dict.get(&callee.to_string()) else {
    return format!("<!-- no command: {:?} -->", text.callee());
  };

  if s == "\\middle" {
    let args_vec: Vec<_> = text.args().items().collect();
    if args_vec.len() != 1 {
      return format!("<!-- unsupported mid arguments: {:?} -->", text.args());
    }
    if let ast::Arg::Pos(expr) = args_vec[0].clone() {
      return format!("\\,\\middle{}\\,", typst_to_mathjax(expr));
    }
    return format!("<!-- unsupported mid argument: {:?} -->", args_vec[0]);
  }

  let args = text
    .args()
    .items()
    .map(|arg| match arg {
      ast::Arg::Pos(value) => format!("{{{}}}", typst_to_mathjax(value)),
      _ => format!("<!-- unsupported argument: {:?} -->", arg),
    })
    .collect::<String>();

  format!("{}{}", s, args)
}

fn analysis_math_text(text: ast::MathText) -> String {
  match text.get() {
    ast::MathTextKind::Character(c) => c.to_string(),
    ast::MathTextKind::Number(n) => n.to_string(),
  }
}

fn analysis_field_access(mut text: ast::FieldAccess) -> String {
  let dict = DICT.get().unwrap();
  let mut fields = vec![text.field().as_str()];

  while let ast::Expr::FieldAccess(next_text) = text.target() {
    text = next_text;
    fields.push(text.field().as_str());
  }
  if let ast::Expr::MathIdent(key) = text.target() {
    if let Some(Value::Object(map)) = dict.get(&key.to_string()) {
      for perm in fields.iter().permutations(fields.len()) {
        let key = perm.iter().map(|&&s| s).collect::<Vec<&str>>().join(".");
        if let Some(Value::String(s)) = map.get(&key) {
          return s.clone();
        }
      }
    }
    return format!("<!-- unsupported field target: {:?} -->", text);
  }

  format!("<!-- fields: {:?} -->", fields)
}

fn analysis_math_indent(text: ast::MathIdent) -> String {
  let dict = DICT.get().unwrap();
  let key = text.to_string();
  match dict.get(&key) {
    Some(Value::String(s)) => s.clone(),
    Some(Value::Object(map)) => {
      if let Some(Value::String(s)) = map.get("main") {
        return s.clone();
      }
      format!("<!-- no command: {:?} -->", text)
    }
    _ => format!("<!-- no command: {:?} -->", text),
  }
}

fn analysis_math_shorthand(text: ast::MathShorthand) -> String {
  text.get().to_string()
}

fn analysis_math_attach(text: ast::MathAttach) -> String {
  let top = text
    .top()
    .map_or(String::new(), |e| format!("^{{{}}}", typst_to_mathjax(e)));
  let bottom = text
    .bottom()
    .map_or(String::new(), |e| format!("_{{{}}}", typst_to_mathjax(e)));
  let base = typst_to_mathjax(text.base());
  let primes = text
    .primes()
    .map_or(String::new(), |e| "'".repeat(e.count()));
  format!("{}{}{}{}", base, primes, top, bottom)
}

fn analysis_math_delimited(text: ast::MathDelimited) -> String {
  let open = match typst_to_mathjax(text.open()).as_str() {
    "{" => "\\{".to_string(),
    s => s.to_string(),
  };
  let close = match typst_to_mathjax(text.close()).as_str() {
    "}" => "\\}".to_string(),
    s => s.to_string(),
  };
  let body = analysis_math(text.body());
  format!("{{\\left{}{}\\right{}}}", open, body, close)
}

fn analysis_math_frac(text: ast::MathFrac) -> String {
  let num = typst_to_mathjax(text.num());
  let denom = typst_to_mathjax(text.denom());
  format!("\\frac{{{}}}{{{}}}", num, denom)
}

// static/ の内容を dist/ にコピー
fn copy_public_assets() -> io::Result<()> {
  for entry in WalkDir::new(STATIC_DIR)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| e.path().is_file())
  {
    let rel_path = entry.path().strip_prefix(STATIC_DIR).unwrap();
    let target_path = Path::new(DIST_DIR).join(rel_path);
    if let Some(parent) = target_path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::copy(entry.path(), &target_path)?;
  }
  Ok(())
}
