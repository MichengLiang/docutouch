# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'DocuTouch 文档系统'
copyright = '2025-2026, 弥澄亮'
author = '弥澄亮'
release = '0.1.0'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = ['sphinx.ext.mathjax',
              'myst_parser',
              'sphinxcontrib.mermaid',
              'sphinxcontrib.bibtex',
              "sphinx_copybutton",    # 代码复制按钮
              "sphinx_design",        # 卡片、栅格、按钮、折叠
              "sphinx_tabs.tabs",     # 代码/内容标签页
              ]

templates_path = ['_templates']

exclude_patterns = ['**/past_records/**', 'build/**']

"""
如果你的目录结构是这样的：

```
docs/
└─ source/
   ├─ index.rst
   ├─ 知识工程/
   │  ├─ intro.rst
   │  └─ past_records/
   │      ├─ 旧文档1.rst
   │      └─ 旧文档2.rst
```

而你希望在构建时排除掉 `source/知识工程/past_records` 整个目录，那么只需要在 `conf.py` 里加上一行：

```python
exclude_patterns = ['知识工程/past_records/*']
```

### ✅ 说明：

* `exclude_patterns` 里的路径是 **相对于 `source` 目录** 的，也就是 Sphinx 的根文档目录。
* 你可以用通配符：

  * `知识工程/past_records/*` → 排除该文件夹下所有文件。
  * `知识工程/past_records/**` → 递归排除所有子目录（如果还有层级）。
  * `**/past_records/**` → 无论在哪个上级目录，只要名字是 `past_records` 都排除。

### 💡例如：

```python
exclude_patterns = [
    '知识工程/past_records/**',
]
```

这就最稳妥了：

> 会递归排除整个 `past_records` 文件夹（包括子目录和文件），而不会影响 `知识工程` 下的其他内容。

"""


language = 'zh_CN'

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_book_theme' # html_theme = 'alabaster' 默认主题
# html_theme = 'alabaster'

html_theme_options = {
    "repository_url": "https://github.com/myrepo/mydocs",
    "use_repository_button": True,
    "use_download_button": True,
    "use_fullscreen_button": True,
    "show_toc_level": 2,
}


# 告诉 Sphinx 在哪里可以找到静态文件（如自定义 CSS）。
# 路径是相对于 conf.py 文件所在目录的。
html_static_path = ['_static']

# 注册需要在 HTML 页面中加载的自定义 CSS 文件列表。
# 文件路径是相对于 html_static_path 中指定的目录的。
html_css_files = [
    'custom.css',
]

source_suffix = {
    '.rst': 'restructuredtext',
    '.md': 'markdown',
}

# -- 启用图表、表格和代码块的自动编号功能 --
numfig = True

myst_enable_extensions = [
    # --- 基础功能 ---
    "deflist",          # 定义列表，用于术语解释等
    "fieldlist",        # 字段列表，用于元数据、参数说明
    "colon_fence",      # ::: 围栏语法，让指令块更美观
    "dollarmath",       # $...$ 和 $$...$$ 数学公式
    "substitution",     # 允许使用 |key| 进行文本替换，实现变量化

    # --- 增强语义与兼容性 ---
    # "attrs_inline",     # 启用内联属性，并增强内联语法解析
    # "substitution",     # 允许使用 |key| 进行文本替换，实现变量化
    # "smartquotes",      # 自动转换标点为印刷体引号（如 “”）

    # --- 实用工具 ---
    # "tasklist",         # 启用 GFM 风格的任务列表 [ ] 和 [x]
]

myst_heading_anchors = 3

myst_substitutions = {
    "project_name": project,
    "project_release": release,
}

bibtex_bibfiles = ['references.bib']
