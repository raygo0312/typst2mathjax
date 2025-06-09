import init, { transform_from_typst } from "./typst2mathjax.js";

// MathJaxの設定
window.MathJax = {
  loader: {
    load: [
      "[tex]/physics",
    ],
  },
  tex: {
    tags: "ams",
    packages: {
      "[+]": ["physics"],
    },
    macros: {
      abs: ["\\left|#1\\right|", 1],
      mid: ["\\middle{#1}", 1],
    }
  },
  chtml: {
    scale: 1,
    mtextInheritFont: true
  },
};

// Typst数式をMathJaxに変換
(async function () {
  await init(); // WASMの初期化

  const textElement = document.querySelector("body");
  const text = textElement.innerHTML;
  const ampText = decodeHtmlEntities(text);
  const result = transform_from_typst(ampText);
  textElement.innerHTML = result;

  const script = document.createElement('script');
  script.src = "https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js";
  script.async = true;
  document.head.appendChild(script);
})();

// &をデコードする関数
function decodeHtmlEntities(text) {
  const textarea = document.createElement('textarea');
  textarea.innerHTML = text;
  return textarea.value;
}