module [View, text, to_str]

View msg := [
    Text Str,
]

text : Str -> View msg
text = |t| @View (Text t)

to_str : View msg -> Str
to_str = |@View (Text t)| t

