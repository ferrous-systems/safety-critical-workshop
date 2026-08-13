// Get Polylux from the official package repository
#import "@preview/polylux:0.4.0": *
#import "../style.typ";

#let template(
  body
) = {
  // Make the paper dimensions fit for a presentation and the text larger
  set page(paper: "presentation-16-9", numbering: "1 / 1", number-align: right)
  set text(size: 20pt, font: style.normal_font)
  show heading: it => [
    #set text(weight: "bold", font: style.normal_font)
    #it
  ]
  show raw: set text(font: style.code_font)

  body
}


#let normal(body, title: str) = {
  slide[
    #set page(
      margin: (top: 80pt),
      header: rect(fill: style.blue, width: 100%, outset: 20%, height: 100%)[
        #set text(fill: style.white, weight: "bold")
        #stack(
          dir: ltr,
          block(width: 100%, stroke: none, [= #title]),
          pad(top: 10pt)[#image("../../assets/images/ferrous-systems/icon/white.svg")]
        )
      ],
    )
    #pad(top: 20pt)[#body]
  ]
}

#let hero(body) = {
  slide[
    #set align(horizon)
    #set page(fill: style.blue)
    #set text(fill: style.white)
    #body
  ]
}

#let title(
  title: str,
  date: datetime.today(),
  author: none,
  affiliation: none,
  logo: none,
) = {
  slide[
    #set align(horizon)
    #set page(fill: style.blue)
    #set text(fill: style.white)
    #stack(
      dir: ltr,
      block(width: 70%, stroke: none)[
        = #title

        #text(fill: teal, weight: "bold")[
          #date.display("[day].[month].[year]")
        ]
        
        #author
        
        #affiliation
      ],
      logo
    )
  ]
}
