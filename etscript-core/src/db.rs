use crate::functions::NResult;
use crate::object::{self, ObjType};
use crate::value::{Value, ValueType};
use crate::vm::Vm;
use crate::{falsey_pattern, truthy_pattern};

use libc::c_longlong;
use rusqlite::{Connection, Result, Statement, ToSql};
use std::{collections::HashMap, path::PathBuf};

extern "C" {
    fn n_system_time_from_unix_time(unix_time: c_longlong) -> *const NResult;
}

pub fn local() -> PathBuf {
    PathBuf::from("./etscript.db")
}

pub fn init() -> Result<()> {
    let db = Connection::open(local())?;

    /*
        db.execute(
            r#"
    CREATE TABLE IF NOT EXISTS _test_attributes (
        memberid           INTEGER  PRIMARY KEY,
        replyemailaddress  TEXT,
        replyname          TEXT,
        member_busname     TEXT,
        member_addr        TEXT,
        member_city        TEXT,
        member_state       TEXT,
        member_postalcode  TEXT,
        member_country     TEXT
    )"#,
            (),
        )?;
        */

    db.execute(
        r#"
CREATE TABLE IF NOT EXISTS _test_table (
    email     TEXT      NOT NULL  PRIMARY KEY,
    text      TEXT,
    number    INTEGER,
    decimal   REAL,
    boolean   BOOLEAN,
    datetime  DATETIME,
    phone     TEXT,
    locale    TEXT
)"#,
        (),
    )?;

    Ok(())
}

//

pub const NO_CASE: &str = "COLLATE NOCASE";

pub struct Table<'a> {
    pub name: &'a str,
    cols: String,
    pub db: Connection,
}

impl<'a> Table<'a> {
    pub fn new(name: &'a str) -> Result<Self, Value> {
        Self::check_name(name, "Table")?;

        let db = Connection::open(local())?;
        Self::table_exists(name, &db)?;
        let cols = Self::get_cols(name, &db)?;

        Ok(Self { name, cols, db })
    }

    fn table_exists(table: &str, db: &Connection) -> Result<(), Value> {
        let mut stmt = db.prepare(&format!(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1 {NO_CASE}"
        ))?;

        let err: Result<String, rusqlite::Error> =
            Err(Value::error("Database error.".to_owned()).into());
        match stmt.query_row([table], |row| row.get(0).or(err)) {
            Ok(_) => String::with_capacity(0),
            Err(_) => return Err(Value::error(format!("No such table: {table}"))),
        };

        Ok(())
    }

    fn get_cols(table: &str, db: &Connection) -> Result<String, Value> {
        // if `get_cols()` ever becomes public, ensure `Self::check_name()` is called prior to `db.prepare()`
        let mut stmt = db.prepare(&format!(
            "SELECT lower(group_concat(name || ':' || type)) FROM pragma_table_info('{table}')"
        ))?;

        let Some(cols): Option<String> = stmt.query_row([], |row| row.get(0))? else {
            return Err(Value::error(format!(
                "Table `{table}` was not found or is not configured."
            )));
        };

        Ok(cols)
    }

    fn check_name(name: &str, of: &str) -> Result<(), Value> {
        if name.is_empty() {
            return Err(Value::error(format!("{of} names cannot be empty.")));
        }
        if name.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(Value::error(format!(
                "{of} names must begin with a letter or underscore."
            )));
        }
        for c in name.chars() {
            if !(c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_') {
                return Err(Value::error(format!(
                    "{of} names must use letters, numbers, or underscores."
                )));
            }
        }

        Ok(())
    }

    fn col_data(&self) -> Result<(Vec<&str>, Vec<&str>), Value> {
        let cols = self.cols.split(',').collect::<Vec<_>>();

        let col_names = cols
            .iter()
            .map(|&col| col.split(':').collect::<Vec<_>>()[0])
            .collect::<Vec<_>>();
        if col_names.is_empty() {
            return Err(Value::error(
                "Column name information is missing.".to_owned(),
            ));
        }

        let col_types = cols
            .iter()
            .map(|&col| col.split(':').collect::<Vec<_>>()[1])
            .collect::<Vec<_>>();
        if col_types.is_empty() {
            return Err(Value::error(
                "Column type information is missing.".to_owned(),
            ));
        }

        Ok((col_names, col_types))
    }

    pub fn col_position(&self, name: &str) -> Result<usize, Value> {
        Self::check_name(name, "Column")?;

        let col_data = self.col_data()?;

        let Some(index) = col_data
            .0
            .iter()
            .position(|&col| col.eq_ignore_ascii_case(name))
        else {
            return Err(Value::error(format!("No such column: {name}")));
        };

        Ok(index)
    }
}

macro_rules! stringy_enum {
    ($vq:vis enum $name:ident {
        $($variant:ident),*$(,)?
    }) => {
        $vq enum $name {
            $($variant),*
        }

        #[allow(unused)]
        impl $name {
            $vq fn as_str(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),*
                }
            }

            $vq fn to_string(&self) -> String {
                match self {
                    $($name::$variant => self.as_str().to_owned()),*
                }
            }

            $vq fn to_upper(&self) -> String {
                match self {
                    $($name::$variant => self.as_str().to_ascii_uppercase()),*
                }
            }

            $vq fn to_lower(&self) -> String {
                match self {
                    $($name::$variant => self.as_str().to_ascii_lowercase()),*
                }
            }
        }
    };
}
pub(crate) use stringy_enum;

stringy_enum! {
    enum SqlType {
        Integer,
        Real,
        Boolean,
        DateTime,
        Text,
    }
}

pub enum AppendType {
    Identifier,
    Predicate,
    Update,
    Upsert,
}

pub fn append_sql_args(
    type_: AppendType,
    sql: &mut String,
    ord_start: usize,
    collation: String,
    arg_range: (usize, usize),
    vm: &Vm,
) -> Result<(), Value> {
    let (mut offset, count) = arg_range;
    let mut ord = ord_start;

    while offset < count {
        let val = &vm.stack[offset];
        if !val.is_string() {
            return Err(Value::error(format!(
                "Name from name-value pair (#{ord}) must be a string."
            )));
        }
        let name = val.to_string(vm);

        match type_ {
            AppendType::Identifier => {
                *sql += &*name;
                if count - offset > 2 {
                    *sql += ",";
                }
            }
            AppendType::Predicate => {
                *sql += &*format!("\nAND {name} = ?{ord} {collation}");
            }
            AppendType::Update => {
                *sql += &*format!(",{name} = ?{ord}");
            }
            AppendType::Upsert => {
                *sql += &*format!(",\n{:<4}{name} = excluded.{name}", "");
            }
        }

        ord += 1;
        offset += 2;
    }

    Ok(())
}

pub fn prep_stmt<'a>(
    table: &'a Table,
    sql: &str,
    param_data: &mut Vec<Box<dyn ToSql>>,
    param_range: (usize, usize),
    vm: &Vm,
) -> Result<Statement<'a>, Value> {
    const NULL: &str = "null";

    let err = |name, type_| {
        Err(Value::error(format!(
            "Unexpected type for column `{name}` — expected `{type_}`."
        )))
    };

    let col_data = &table.col_data()?;
    let mut col_ord = 0;
    let (mut offset, count) = param_range;
    let mut i = 0;

    while i < count {
        let val = &vm.stack[offset];

        if i % 2 == 0 {
            if !val.is_string() {
                return Err(Value::error(format!(
                    "Column #{}'s name must be a string.",
                    i / 2
                )));
            }

            let name = val.to_string(vm);

            col_ord = table.col_position(&name)?;
        } else {
            let col_name = col_data.0[col_ord];
            let col_type = col_data.1[col_ord];

            match &val.type_ {
                ValueType::Null(_) => {
                    param_data.push(Box::new(NULL));
                }
                ValueType::Number(num_f64) => match col_type {
                    t if t == SqlType::Boolean.to_lower() => match *num_f64 {
                        n if n == 1f64 => param_data.push(Box::new(true)),
                        n if n == 0f64 => param_data.push(Box::new(false)),
                        _ => err(col_name, col_type)?,
                    },
                    _ => {
                        if !(col_type == SqlType::Integer.to_lower()
                            || col_type == SqlType::Real.to_lower())
                        {
                            err(col_name, col_type)?;
                        }
                        let mut num = *num_f64;

                        if col_type == SqlType::Integer.to_lower() {
                            num = f64::trunc(*num_f64);
                        }
                        param_data.push(Box::new(num));
                    }
                },
                ValueType::Boolean(boolean) => {
                    if col_type != SqlType::Boolean.to_lower() {
                        err(col_name, col_type)?;
                    }
                    param_data.push(Box::new(*boolean));
                }
                ValueType::Obj(obj) => match &obj.type_ {
                    ObjType::String(string_obj) => {
                        let string = string_obj.get(&vm.strings.borrow()).to_ascii_lowercase();
                        match col_type {
                            t if t == SqlType::Boolean.to_lower() => match string.as_str() {
                                truthy_pattern!() => param_data.push(Box::new(true)),
                                falsey_pattern!() => param_data.push(Box::new(false)),
                                _ => err(col_name, col_type)?,
                            },
                            t if t == SqlType::DateTime.to_lower() => {
                                if !string_obj.is_datetime {
                                    err(col_name, col_type)?;
                                } else {
                                    param_data.push(Box::new(string_obj.data));
                                }
                            }
                            _ => {
                                if col_type != SqlType::Text.to_lower() {
                                    err(col_name, col_type)?;
                                }
                                param_data.push(Box::new(string));
                            }
                        }
                    }
                    _ => param_data.push(Box::new(NULL)),
                },
            }
        }

        offset += 1;
        i += 1;
    }

    // table and column names are valid; `ToSql` parameter conversion complete
    Ok(table.db.prepare(sql)?)
}

pub fn exec_stmt(
    mut stmt: Statement,
    params: &[&dyn ToSql],
    vm: &mut Vm,
) -> Result<Vec<Value>, Value> {
    let col_count = stmt.column_count();
    let keys = stmt
        .column_names()
        .into_iter()
        .map(|name| {
            let string = name.to_ascii_lowercase();
            object::intern(string, vm)
        })
        .collect::<Vec<_>>();

    let rows = stmt.query_map(params, |row| {
        let types: String = row.get(col_count - 1)?;
        let mut types_iter = types.split(',');
        let mut map = HashMap::new();

        for (i, key) in keys.iter().enumerate().take(col_count - 1) {
            match types_iter.next() {
                Some("integer" | "real") => {
                    let val = if let Some(sql_val) = row.get(i).unwrap_or(None) {
                        Value::num(sql_val)
                    } else {
                        Value::null()
                    };
                    map.insert(*key, val)
                }
                Some("boolean") => {
                    let val = if let Some(sql_val) = row.get(i).unwrap_or(None) {
                        Value::boolean(sql_val)
                    } else {
                        Value::null()
                    };
                    map.insert(*key, val)
                }
                Some("datetime") => {
                    let val = if let Some(sql_val) = row.get(i).unwrap_or(None) {
                        unsafe {
                            let dt = NResult::consume(n_system_time_from_unix_time(sql_val))?;
                            Value::datetime(dt, vm)
                        }
                    } else {
                        Value::null()
                    };
                    map.insert(*key, val)
                }
                Some("text") => {
                    let val = if let Some(sql_val) = row.get(i).unwrap_or(None) {
                        Value::string(sql_val, vm)
                    } else {
                        Value::null()
                    };
                    map.insert(*key, val)
                }
                _ => map.insert(*key, Value::null()),
            };
        }

        Ok(Value::row(map))
    })?;

    // `Vec<Value>` where `Value` is `ValueType::Obj` and `Obj` is `ObjType::Row`
    let mut row_vals = Vec::new(); //

    // some queries are expected to return `ValueType::Null`, so construction of
    // `ObjType::Rowset` takes place in `exec_stmt`'s caller instead
    for row_val in rows {
        row_vals.push(row_val?);
    }

    Ok(row_vals)
}

impl From<rusqlite::Error> for Value {
    fn from(err: rusqlite::Error) -> Self {
        // todo: Some SQLite errors include too much info (e.g., raw queries).
        // Look into matching against:
        // https://docs.rs/rusqlite/latest/rusqlite/enum.Error.html
        Value::error(err.to_string())
    }
}

impl From<Value> for rusqlite::Error {
    fn from(_val: Value) -> Self {
        rusqlite::Error::QueryReturnedNoRows
    }
}
