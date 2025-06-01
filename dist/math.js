MathJax = {
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