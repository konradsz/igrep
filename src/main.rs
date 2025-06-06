use anyhow::Result;
use igrep::{
    app::App,
    args::Args,
    editor::EditorCommand,
    ig,
    ui::{
        context_viewer::ContextViewer,
        theme::{dark::Dark, light::Light, Theme, ThemeVariant},
    },
};
use std::io::Write;

fn main() -> Result<()> {
    let args = Args::parse_cli_and_config_file();

    if args.type_list {
        use itertools::Itertools;
        let mut builder = ignore::types::TypesBuilder::new();
        builder.add_defaults();
        for definition in builder.definitions() {
            writeln!(
                std::io::stdout(),
                "{}: {}",
                definition.name(),
                definition.globs().iter().format(", "),
            )?;
        }
        return Ok(());
    }

    let paths = if args.paths.is_empty() {
        vec!["./".into()]
    } else {
        args.paths
    };

    let search_config = ig::SearchConfig::from(args.pattern.unwrap(), paths)?
        .case_insensitive(args.ignore_case)
        .case_smart(args.smart_case)
        .search_hidden(args.search_hidden)
        .follow_links(args.follow_links)
        .word_regexp(args.word_regexp)
        .globs(args.glob)?
        .file_types(args.type_matching, args.type_not)?
        .sort_by(args.sort_by, args.sort_by_reverse)?;

    let theme: Box<dyn Theme> = match args.theme {
        ThemeVariant::Light => Box::new(Light),
        ThemeVariant::Dark => Box::new(Dark),
    };
    let mut app = App::new(
        search_config,
        EditorCommand::new(args.editor.custom_command, args.editor.editor)?,
        ContextViewer::new(args.context_viewer),
        theme,
    );
    app.run()?;

    Ok(())
}
