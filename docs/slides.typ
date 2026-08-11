
#import "../lib.typ": slides;

#show: slides.template.with()

#slides.title(
  title: [A demo of slides],
  date: datetime.today(),
  author: [Manuel Hatzl & Alex Senier],
  affiliation: [Ferrous Systems GmbH],
  logo: image("../assets/images/ferrous-systems/icon/white.svg"),
)

#slides.normal(title: [Overview])[
  1. Talk about stuff
  2. Other stuff too!
]

#slides.normal(title: [A code example])[
  *Rust:*
  ```rust
  let x = y;
  fn foo() {
    
  }
  ```

  *Toml:*
  ```toml
  [Foo.bar]
  baz.bat = "potato"
  ```
]

#slides.normal(title: [A math example])[
  Some static text on this slide.

  $frac(integral(x^2) - delta(x_2), delta(y_2) + integral(y^2))$
]

#slides.normal(title: [Quarter 1])[
  - Beans
  - Waffles
  - Potatoes
]

#slides.normal(title: [Quarter 2])[
  You can always see this.

  #slides.uncover(2)[But this appears later!]
]

#slides.hero[
  = Contact

  #text(fill: teal, weight: "bold")[
    manuel.hatzl\@ferrous-systems.com
  ]

  #text(fill: teal, weight: "bold")[
    alex.senier\@ferrous-systems.com
  ]
]
