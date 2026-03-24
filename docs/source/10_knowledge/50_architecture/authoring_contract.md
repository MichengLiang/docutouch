(knowledge-architecture-contract)=
# 50 Architecture 作者契约

## 契约范围

本页裁定 accepted architecture description 应如何在本目录落位。

## Allowed Objects

- architecture view
- solution strategy
- runtime / deployment structure
- crosscutting architecture concept

## Disallowed Objects

- accepted decision record
- interface contract
- operational procedure
- candidate architecture proposal

## Dependency Discipline

- architecture 以前置 requirements、principles、problem-space 为上游
- architecture 页可 cross-reference decision 页，但 decision 不应成为 architecture 的唯一 authority source
- 若对象主要是对外 contract，应迁入 `70_interfaces/`

