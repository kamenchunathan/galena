module [
    Html,
    Attr,
    # HTML elements
    text,
    div,
    p,
    span,
    h1,
    h2,
    h3,
    h4,
    h5,
    h6,
    a,
    img,
    button,
    input,
    form,
    ul,
    ol,
    li,
    section,
    article,
    header,
    footer,
    nav,
    main,
    aside,
    strong,
    em,
    code,
    pre,
    table,
    tr,
    td,
    th,
    thead,
    tbody,
    tfoot,
    select,
    option,
    textarea,
    label,
    br,
    hr,
    # Attributes
    id,
    class,
    value,
    placeholder,
    type,
    name,
    href,
    src,
    alt,
    title,
    disabled,
    checked,
    selected,
    hidden,
    readonly,
    required,
    multiple,
    autocomplete,
    tabindex,
    style_attr,
    data,
    # Event handlers
    on_click,
    on_input,
    on_change,
    on_submit,
    on_focus,
    on_blur,
    on_keydown,
    on_keyup,
    on_keypress,
    on_mousedown,
    on_mouseup,
    on_mouseover,
    on_mouseout,
    on_mouseenter,
    on_mouseleave,
    on_load,
    on_scroll,
]

import Internal.Html exposing [
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

import Internal.Attr exposing [
    InternalAttr,
    id_,
    class_,
    value_,
    placeholder_,
    type_,
    name_,
    href_,
    src_,
    alt_,
    title_,
    disabled_,
    checked_,
    selected_,
    hidden_,
    readonly_,
    required_,
    multiple_,
    autocomplete_,
    tabindex_,
    style_,
    data_,
    on_click_,
    on_input_,
    on_change_,
    on_submit_,
    on_focus_,
    on_blur_,
    on_keydown_,
    on_keyup_,
    on_keypress_,
    on_mousedown_,
    on_mouseup_,
    on_mouseover_,
    on_mouseout_,
    on_mouseenter_,
    on_mouseleave_,
    on_load_,
    on_scroll_,
]

# Type aliases
Html msg : InternalHtml msg
Attr msg : InternalAttr msg

# HTML elements
text = text_
div = div_
p = p_
span = span_

# Headings
h1 = h1_
h2 = h2_
h3 = h3_
h4 = h4_
h5 = h5_
h6 = h6_

# Text formatting
strong = strong_
em = em_
code = code_
pre = pre_

# Links and media
a = a_
img = img_

# Forms
form = form_
input = input_
button = button_
select = select_
option = option_
textarea = textarea_
label = label_

# Lists
ul = ul_
ol = ol_
li = li_

# Structural elements
section = section_
article = article_
header = header_
footer = footer_
nav = nav_
main = main_
aside = aside_

# Tables
table = table_
tr = tr_
td = td_
th = th_
thead = thead_
tbody = tbody_
tfoot = tfoot_

# Self-closing elements
br = br_
hr = hr_

# Standard attributes
id = id_
class = class_
value = value_
placeholder = placeholder_
type = type_
name = name_
href = href_
src = src_
alt = alt_
title = title_

# Boolean attributes
disabled = disabled_
checked = checked_
selected = selected_
hidden = hidden_
readonly = readonly_
required = required_
multiple = multiple_

# Other attributes
autocomplete = autocomplete_
tabindex = tabindex_
style_attr = style_ # Renamed to avoid conflict with style element
data = data_

# Event handlers
on_click = on_click_
on_input = on_input_
on_change = on_change_
on_submit = on_submit_
on_focus = on_focus_
on_blur = on_blur_
on_keydown = on_keydown_
on_keyup = on_keyup_
on_keypress = on_keypress_
on_mousedown = on_mousedown_
on_mouseup = on_mouseup_
on_mouseover = on_mouseover_
on_mouseout = on_mouseout_
on_mouseenter = on_mouseenter_
on_mouseleave = on_mouseleave_
on_load = on_load_
on_scroll = on_scroll_
