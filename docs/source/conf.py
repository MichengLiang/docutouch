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

language = 'zh'

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
