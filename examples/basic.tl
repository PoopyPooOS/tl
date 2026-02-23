with {
  types.port = |x: uint| x <= 65535
  greet = |name: string|: string "Hello, ${name}!"
}
[
  types.port(80)
  greet("world")
]
