# TL

TL probably meant something but I forgot  
A small interpreted programing language built to be a config language.

A simple interpreted language, and by simple I mean broken and lacking in features.

## Features
- `serde`: This feature lets you deserialize a [`Value`](src/runtime/types.rs) enum into a rust type. It also exposes [`tl::eval<T: Deserialize>`](src/utils.rs) and [`tl::eval_untyped`](src/utils.rs).
