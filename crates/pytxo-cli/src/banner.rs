//! The merge λ beside the wordmark, printed above `pytxo --help` in a terminal.

use clap::builder::styling::{Ansi256Color, Effects, Reset, Style};

/// `{m}` starts the magenta node, `{d}` the dimmed shadow and tagline, `{/}` ends either.
/// The node touches the strokes on purpose: at 16 half-block rows the mark's knockout ring
/// splits the λ into a head and two feet, so here the colour alone sets the node apart.
const BANNER: &str = r#"▀███▄
 ▀███▄        ██████{d}╗{/} ██{d}╗{/}   ██{d}╗{/}████████{d}╗{/}██{d}╗{/}  ██{d}╗{/} ██████{d}╗{/}
  ▀███▄       ██{d}╔══{/}██{d}╗╚{/}██{d}╗{/} ██{d}╔╝╚══{/}██{d}╔══╝╚{/}██{d}╗{/}██{d}╔╝{/}██{d}╔═══{/}██{d}╗{/}
   ▀███▄      ██████{d}╔╝{/} {d}╚{/}████{d}╔╝{/}    ██{d}║{/}    {d}╚{/}███{d}╔╝{/} ██{d}║{/}   ██{d}║{/}
   ▄{m}▄██▄{/}▄     ██{d}╔═══╝{/}   {d}╚{/}██{d}╔╝{/}     ██{d}║{/}    ██{d}╔{/}██{d}╗{/} ██{d}║{/}   ██{d}║{/}
  ▄█{m}▀██▀{/}█▄    ██{d}║{/}        ██{d}║{/}      ██{d}║{/}   ██{d}╔╝{/} ██{d}╗╚{/}██████{d}╔╝{/}
 ▄███▀▀███▄   {d}╚═╝{/}        {d}╚═╝{/}      {d}╚═╝{/}   {d}╚═╝{/}  {d}╚═╝{/} {d}╚═════╝{/}
▄███▀  ▀███▄  {d}the landing layer for coding agents{/}"#;

/// The banner with its colours as ANSI escapes, which clap drops when NO_COLOR is set.
pub fn render() -> String {
    // 205 is the 256-colour cube's nearest to the logo's #F04DA3: more terminals
    // show it than 24-bit colour.
    let node = Style::new().fg_color(Some(Ansi256Color(205).into()));
    let dim = Style::new().effects(Effects::DIMMED);
    BANNER
        .replace("{m}", &node.render().to_string())
        .replace("{d}", &dim.render().to_string())
        .replace("{/}", &Reset.render().to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn help_prints_the_banner_as_drawn() {
        let plain = super::BANNER
            .replace("{m}", "")
            .replace("{d}", "")
            .replace("{/}", "");
        assert!(!plain.contains('{'), "unknown marker in the banner");
        assert!(plain.lines().all(|line| line.chars().count() <= 60));
        // clap keeps every cell: no wrapping, no collapsed spaces, every escape resolved.
        let help = clap::Command::new("pytxo")
            .before_help(super::render())
            .render_help()
            .to_string();
        assert!(help.starts_with(&plain), "{help}");
    }
}
