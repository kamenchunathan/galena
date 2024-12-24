module [View, text]

View msg := {}

text : Str -> View msg
text = \_ -> @View {}

