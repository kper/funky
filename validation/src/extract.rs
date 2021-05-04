use wasm_parser::core::*;
use wasm_parser::Module;

pub(crate) fn get_types(module: &Module) -> Vec<&FunctionSignature> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Type(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    ty
}

pub fn get_exports(module: &Module) -> Vec<&ExportEntry> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Export(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    ty
}

pub fn get_imports(module: &Module) -> Vec<&ImportEntry> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Import(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    ty
}

pub fn get_start(module: &Module) -> Vec<&StartSection> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Start(t) => Some(t),
            _ => None,
        })
        .collect();

    ty
}

pub fn get_elements(module: &Module) -> Vec<&ElementSegment> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Element(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    ty
}

pub fn get_data(module: &Module) -> Vec<&DataSegment> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Data(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    ty
}

pub fn get_funcs(module: &Module) -> Vec<FuncIdx> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Function(t) => Some(t.types.clone()),
            Section::Import(section) => {
                let entries = section
                    .entries
                    .iter()
                    .filter_map(|entry| match &entry.desc {
                        ImportDesc::Function { ty: k } => Some(k.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                Some(entries)
            }
            _ => None,
        })
        .flatten()
        .map(|x| x.clone())
        .collect();

    ty
}

pub fn get_defined_tables(module: &Module) -> Vec<&TableType> {
    module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Table(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect()
}

pub fn get_tables(module: &Module) -> Vec<&TableType> {
    let ty = get_defined_tables(module);

    let imported: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Import(section) => {
                let entries = section
                    .entries
                    .iter()
                    .filter_map(|entry| match &entry.desc {
                        ImportDesc::Table { ty: k } => Some(k),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                Some(entries)
            }
            _ => None,
        })
        .flatten()
        .collect();

    let mut all = Vec::with_capacity(ty.len() + imported.len());
    all.extend(ty);
    all.extend(imported);

    all
}

pub fn get_mems(module: &Module) -> Vec<&MemoryType> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Memory(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    let imported: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Import(section) => {
                let entries = section
                    .entries
                    .iter()
                    .filter_map(|entry| match &entry.desc {
                        ImportDesc::Memory { ty: k } => Some(k),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                Some(entries)
            }
            _ => None,
        })
        .flatten()
        .collect();

    let mut all = Vec::with_capacity(ty.len() + imported.len());
    all.extend(ty);
    all.extend(imported);

    all
}

pub fn get_globals(module: &Module) -> (Vec<&GlobalVariable>, Vec<&GlobalType>) {
    let ty = get_defined_globals(module);
    let imported = get_imported_globals(module)
        .into_iter()
        .filter_map(|entry| match &entry.desc {
            ImportDesc::Global { ty: k } => Some(k),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut all = Vec::with_capacity(ty.len() + imported.len());
    all.extend(ty.iter().map(|w| &w.ty).collect::<Vec<&GlobalType>>());
    all.extend(imported);

    (ty, all)
}

pub fn get_defined_globals(module: &Module) -> Vec<&GlobalVariable> {
    let ty: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Global(t) => Some(&t.globals),
            _ => None,
        })
        .flatten()
        .collect();

    ty
}

pub fn get_imported_globals(module: &Module) -> Vec<&ImportEntry> {
    let imported: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Import(section) => {
                let entries = section
                    .entries
                    .iter()
                    .filter(|x| matches!(&x.desc, ImportDesc::Global { .. }))
                    /*
                    .filter_map(|entry| match &entry.desc {
                        ImportDesc::Global { ty } => Some()),
                        _ => None,
                    })*/
                    .collect::<Vec<_>>();

                Some(entries)
            }
            _ => None,
        })
        .flatten()
        .collect();

    imported
}
