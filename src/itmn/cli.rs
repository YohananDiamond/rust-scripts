use clap::Clap;

#[derive(Clap)]
pub struct Options {
    #[clap(
        short,
        long,
        about = "The path to the entries file (default: $ITMN_FILE => ~/.local/share/itmn)"
    )]
    pub path: Option<String>,
    #[clap(subcommand, about = "The command to be ran - defaults to [next]")]
    pub subcmd: Option<SubCmd>,
}

#[derive(Clap)]
pub enum SubCmd {
    // #[clap(subcommand, about = "Shows a report - defaults to [next]")]
    // TODO: Report(ReportSelection),
    #[clap(alias = "ls", about = "An alias to the [except-done] report")]
    List,
    #[clap(about = "An alias to the [next] report")]
    Next,
    #[clap(about = "Add an item")]
    Add(ItemAddDetails),
    #[clap(alias = "sel", about = "Select items by ID and do something with them")]
    SelRefID(SelectionDetails),
    // TODO: SelInternalID(SelectionDetails),
    // TODO: Search,
    // TODO: RegexMatch,
}

#[derive(Clap)]
pub struct ItemAddDetails {
    #[clap(about = "The name of the item")]
    pub name: String,
    #[clap(short, long, about = "The context of the item")]
    pub context: Option<String>,
    #[clap(short, long, about = "If the item is a note")]
    pub note: Option<bool>,
}

#[derive(Clap)]
pub struct SelectionDetails {
    #[clap(about = "The selection range")]
    pub range: String, // TODO: document range syntax
    #[clap(
        subcommand,
        about = "What to do with the selection, defaults to [list-tree]"
    )]
    pub action: Option<SelectionAction>,
}

#[derive(Clap)]
pub enum SelectionAction {
    #[clap(alias = "mod", about = "Modify the matches")]
    Modify(ItemBatchMod),
    #[clap(aliases = &["sub"], about = "Add a child to each one of the matches")]
    AddChild(ItemAddDetails), // TODO: require confirmation if the amount of items selected is more than one.
    #[clap(about = "Mark the matches as DONE, if their states are TODO")]
    Done,
    #[clap(alias = "tree", about = "List matches in a tree")]
    ListTree,
    #[clap(aliases = &["ls", "list"], about = "List matches, showing only the first child of each, if any")]
    ListBrief,
    #[clap(about = "List matches without showing any children")]
    ListShallow,
    // TODO: Delete(DelArgs), // TODO: --force/-f option
}

#[derive(Clap)]
pub struct ItemBatchMod {
    #[clap(about = "The item's new name")]
    pub name: Option<String>,
    #[clap(short, long, about = "The item's new context; set to an empty string to unset")]
    pub context: Option<String>,
    #[clap(short, long, about = "The item's new type")]
    pub note: Option<bool>,
}