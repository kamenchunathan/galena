module [
    InternalHtml,
    text_,
    div_,
    p_,
    span_,
    h1_,
    h2_,
    h3_,
    h4_,
    h5_,
    h6_,
    a_,
    img_,
    button_,
    input_,
    form_,
    ul_,
    ol_,
    li_,
    section_,
    article_,
    header_,
    footer_,
    nav_,
    main_,
    aside_,
    strong_,
    em_,
    code_,
    pre_,
    table_,
    tr_,
    td_,
    th_,
    thead_,
    tbody_,
    tfoot_,
    select_,
    option_,
    textarea_,
    label_,
    br_,
    hr_,
]

import Internal.Attr exposing [InternalAttr]

InternalHtml msg := [
    Text Str,
    Element
        {
            tag : Str,
            attrs : List (InternalAttr msg),
            children : List (InternalHtml msg),
        },
]

# Helper to create elements
createElement : Str, List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
createElement = |tag, attrs, children|
    @InternalHtml
        (
            Element {
                tag: tag,
                attrs: attrs,
                children: children,
            }
        )

# Text node
text_ : Str -> InternalHtml msg
text_ = |t| @InternalHtml (Text t)

# Basic elements
div_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
div_ = |attrs, children| createElement "div" attrs children

p_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
p_ = |attrs, children| createElement "p" attrs children

span_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
span_ = |attrs, children| createElement "span" attrs children

# Headings
h1_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
h1_ = |attrs, children| createElement "h1" attrs children

h2_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
h2_ = |attrs, children| createElement "h2" attrs children

h3_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
h3_ = |attrs, children| createElement "h3" attrs children

h4_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
h4_ = |attrs, children| createElement "h4" attrs children

h5_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
h5_ = |attrs, children| createElement "h5" attrs children

h6_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
h6_ = |attrs, children| createElement "h6" attrs children

# Text formatting
strong_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
strong_ = |attrs, children| createElement "strong" attrs children

em_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
em_ = |attrs, children| createElement "em" attrs children

code_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
code_ = |attrs, children| createElement "code" attrs children

pre_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
pre_ = |attrs, children| createElement "pre" attrs children

# Links and media
a_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
a_ = |attrs, children| createElement "a" attrs children

img_ : List (InternalAttr msg) -> InternalHtml msg
img_ = |attrs| createElement "img" attrs []

# Forms
form_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
form_ = |attrs, children| createElement "form" attrs children

input_ : List (InternalAttr msg) -> InternalHtml msg
input_ = |attrs| createElement "input" attrs []

button_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
button_ = |attrs, children| createElement "button" attrs children

select_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
select_ = |attrs, children| createElement "select" attrs children

option_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
option_ = |attrs, children| createElement "option" attrs children

textarea_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
textarea_ = |attrs, children| createElement "textarea" attrs children

label_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
label_ = |attrs, children| createElement "label" attrs children

# Lists
ul_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
ul_ = |attrs, children| createElement "ul" attrs children

ol_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
ol_ = |attrs, children| createElement "ol" attrs children

li_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
li_ = |attrs, children| createElement "li" attrs children

# Structural elements
section_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
section_ = |attrs, children| createElement "section" attrs children

article_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
article_ = |attrs, children| createElement "article" attrs children

header_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
header_ = |attrs, children| createElement "header" attrs children

footer_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
footer_ = |attrs, children| createElement "footer" attrs children

nav_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
nav_ = |attrs, children| createElement "nav" attrs children

main_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
main_ = |attrs, children| createElement "main" attrs children

aside_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
aside_ = |attrs, children| createElement "aside" attrs children

# Tables
table_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
table_ = |attrs, children| createElement "table" attrs children

tr_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
tr_ = |attrs, children| createElement "tr" attrs children

td_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
td_ = |attrs, children| createElement "td" attrs children

th_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
th_ = |attrs, children| createElement "th" attrs children

thead_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
thead_ = |attrs, children| createElement "thead" attrs children

tbody_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
tbody_ = |attrs, children| createElement "tbody" attrs children

tfoot_ : List (InternalAttr msg), List (InternalHtml msg) -> InternalHtml msg
tfoot_ = |attrs, children| createElement "tfoot" attrs children

# Self-closing elements
br_ : List (InternalAttr msg) -> InternalHtml msg
br_ = |attrs| createElement "br" attrs []

hr_ : List (InternalAttr msg) -> InternalHtml msg
hr_ = |attrs| createElement "hr" attrs []
