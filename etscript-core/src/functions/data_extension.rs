use crate::db::{self, AppendType, Table, NO_CASE};
use crate::functions::{self, ArgType};
use crate::object;
use crate::value::Value;
use crate::vm::Vm;

use crate::object::StringObj;
use rusqlite::{Result, ToSql};

pub fn insert_de(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::String, // insert column name
        ArgType::Value,  // insert column value
                         // (insert column name/value ...)
                         // -> Null
    ];

    exec_insert(Insert::Send, ARITY_MIN, ARG_TYPES, arg_start, arg_count, vm)
}

pub fn insert_data(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::String, // insert column name
        ArgType::Value,  // insert column value
                         // (insert column name/value ...)
                         // -> Number
    ];

    exec_insert(Insert::Req, ARITY_MIN, ARG_TYPES, arg_start, arg_count, vm)
}

pub fn lookup(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 4;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::String, // result column name
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
                         // (filter column name/value ...)
                         // -> Value/Null
    ];

    exec_select(Select::Row, ARITY_MIN, ARG_TYPES, arg_start, arg_count, vm)
}

pub fn lookup_rows(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
                         // (filter column name/value ...)
                         // -> Rowset
    ];

    exec_select(
        Select::Rowset,
        ARITY_MIN,
        ARG_TYPES,
        arg_start,
        arg_count,
        vm,
    )
}

pub fn lookup_rows_cs(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
                         // (filter column name/value ...)
                         // -> Rowset
    ];

    exec_select(
        Select::RowsetCs,
        ARITY_MIN,
        ARG_TYPES,
        arg_start,
        arg_count,
        vm,
    )
}

pub fn lookup_ordered_rows(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 5;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::Number, // limit
        ArgType::String, // result column name + order
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
                         // (filter column name/value ...)
                         // -> Rowset
    ];

    exec_select(
        Select::Ordered,
        ARITY_MIN,
        ARG_TYPES,
        arg_start,
        arg_count,
        vm,
    )
}

pub fn lookup_ordered_rows_cs(
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 5;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::Number, // limit
        ArgType::String, // result column name + order
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
                         // (filter column name/value ...)
                         // -> Rowset
    ];

    exec_select(
        Select::OrderedCs,
        ARITY_MIN,
        ARG_TYPES,
        arg_start,
        arg_count,
        vm,
    )
}

pub fn update_de(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 6;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::Number, // filter column count
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
        //                  (filter column name/value ...)
        ArgType::String, // update column name
        ArgType::Value,  // update column value
                         // (update column name/value ...)
                         // -> Null
    ];

    exec_update(Update::Send, ARITY_MIN, ARG_TYPES, arg_start, arg_count, vm)
}

pub fn update_data(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 6;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::Number, // filter column count
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
        //                  (filter column name/value ...)
        ArgType::String, // update column name
        ArgType::Value,  // update column value
                         // (update column name/value ...)
                         // -> Number
    ];

    exec_update(Update::Req, ARITY_MIN, ARG_TYPES, arg_start, arg_count, vm)
}

pub fn upsert_de(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 6;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::Number, // filter column count
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
        //                  (filter column name/value ...)
        ArgType::String, // update column name
        ArgType::Value,  // update column value
                         // (update column name/value ...)
                         // -> Number
    ];

    exec_update(
        Update::UpsertSend,
        ARITY_MIN,
        ARG_TYPES,
        arg_start,
        arg_count,
        vm,
    )
}

pub fn upsert_data(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 6;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::Number, // filter column count
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
        //                  (filter column name/value ...)
        ArgType::String, // update column name
        ArgType::Value,  // update column value
                         // (update column name/value ...)
                         // -> Number
    ];

    exec_update(
        Update::UpsertReq,
        ARITY_MIN,
        ARG_TYPES,
        arg_start,
        arg_count,
        vm,
    )
}

pub fn delete_de(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
                         // (filter column name/value ...)
                         // -> Null
    ];

    exec_delete(Delete::Send, ARITY_MIN, ARG_TYPES, arg_start, arg_count, vm)
}

pub fn delete_data(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
        ArgType::String, // filter column name
        ArgType::Value,  // filter column value
                         // (filter column name/value ...)
                         // -> Number
    ];

    exec_delete(Delete::Req, ARITY_MIN, ARG_TYPES, arg_start, arg_count, vm)
}

//

pub fn field(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 2;
    const ARITY_MAX: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Row, // target
        ArgType::Value, // key
                      // (error if key undefined): Boolean
                      // -> Value/Null
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let row = stack[arg_start].to_row();
    let name = &stack[arg_start + 1];
    let string;
    let key = if name.is_string() {
        let obj: StringObj = (*name).clone().into();
        string = obj.get(&vm.strings.borrow()).to_owned();
        obj.hash
    } else if name.is_num() {
        let num = name.to_num(vm);
        string = format!("{num}");
        object::hash_of(&string)
    } else {
        return Err(Value::error(
            "Unexpected type passed to function — expected a string or number.".to_owned(),
        ));
    };

    let should_err = if arg_count == ARITY_MAX {
        let offset = arg_start + 2;
        functions::check_arg_types(&[ArgType::Boolean], offset, vm)?;
        stack[offset].to_bool(vm)
    } else {
        true
    };

    let val = if let Some(val) = row.map.get(&key) {
        (*val).clone()
    } else if should_err {
        return Err(Value::error(format!(
            "Field name `{string}` was not found."
        )));
    } else {
        Value::null()
    };

    Ok(val)
}

pub fn row(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Rowset, // target
        ArgType::Number, // index (1-based)
                         // -> Row
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let rowset = stack[arg_start].to_rowset();
    let count = rowset.vec.len();
    let index = f64::trunc(stack[arg_start + 1].to_num(vm)) as usize;

    if count < 1 {
        return Err(Value::error("Rowset is empty.".to_owned()));
    }
    if !(1..=count).contains(&index) {
        return Err(Value::error(format!(
            "Row {index} is out of range. Rowset has a row count of {count}."
        )));
    }

    Ok(rowset.vec[index - 1].clone())
}

pub fn row_count(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Rowset, // target
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let rowset = stack[arg_start].to_rowset();

    Ok(Value::num(rowset.vec.len() as f64))
}

pub fn data_extension_row_count(
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // table
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let tbl_name = stack[arg_start].to_ascii_lower(vm);
    let table = Table::new(&tbl_name)?;

    // table name already validated
    let mut stmt = table
        .db
        .prepare(&format!("SELECT count(rowid) FROM {tbl_name}"))?;

    let Some(count): Option<f64> = stmt.query_row([], |row| row.get(0))? else {
        return Err(Value::error(format!(
            "Table `{}` was not found or is not configured.", table.name
        )));
    };

    Ok(Value::num(count))
}

//

#[derive(PartialEq, Eq)]
enum Insert {
    Send,
    Req,
}

#[derive(PartialEq, Eq)]
enum Select {
    Row,
    Rowset,
    RowsetCs,
    Ordered,
    OrderedCs,
}

#[derive(PartialEq, Eq)]
enum Update {
    Send,
    Req,
    UpsertSend,
    UpsertReq,
}

#[derive(PartialEq, Eq)]
enum Delete {
    Send,
    Req,
}

fn check_arg_pairs(arity_min: u8, arg_count: u8) -> Result<(), Value> {
    if arg_count <= arity_min {
        return Ok(()); // additional arguments were not passed
    }
    if (arg_count - arity_min) % 2 != 0 {
        return Err(Value::error(
            "Additional arguments passed to function are unbalanced — \
                extended clauses consist of name-value pairs."
                .to_owned(),
        ));
    }

    Ok(())
}

fn exec_insert(
    ins_context: Insert,
    arity_min: u8,
    arg_types: &[ArgType],
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    let stack = &vm.stack;
    functions::check_arity_min(arity_min, arg_count)?;
    functions::check_arg_types(arg_types, arg_start, vm)?;
    check_arg_pairs(arity_min, arg_count)?;

    let tbl_name = stack[arg_start].to_ascii_lower(vm);
    let table = Table::new(&tbl_name)?;

    let insert_col_offset = arg_start + 1;
    let insert_col_count = (stack.len() - insert_col_offset) / 2;

    // table and column names are validated prior to prepared statement creation
    let mut sql = format!(
        r"
INSERT INTO {tbl_name} ("
    );

    let collation = String::with_capacity(0); // unused
    let ord_start = 1;
    let start = insert_col_offset;

    db::append_sql_args(
        AppendType::Identifier,
        &mut sql,
        ord_start,
        collation,
        (start, stack.len()),
        vm,
    )?;
    sql += ") VALUES (";

    let mut ord = 1;
    let mut i = 0;
    while i < insert_col_count {
        if i > 0 {
            sql += ",";
        }
        sql += &format!("?{ord}");
        ord += 1;
        i += 1;
    }
    sql += ")";

    let mut param_data: Vec<Box<dyn ToSql>> = Vec::new();
    let param_count = stack.len() - insert_col_offset;

    let mut stmt = db::prep_stmt(
        &table,
        &sql,
        &mut param_data,
        (insert_col_offset, param_count),
        vm,
    )?;

    let params = param_data
        .iter()
        .map(|val| val.as_ref())
        .collect::<Vec<_>>();
    let rows_affected = stmt.execute(params.as_slice())?;

    match ins_context {
        Insert::Send => Ok(Value::null()),
        Insert::Req => Ok(Value::num(rows_affected as f64)),
    }
}

fn exec_select(
    sel_context: Select,
    arity_min: u8,
    arg_types: &[ArgType],
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    const RESULT_MAX: usize = 2000;
    let stack = &vm.stack;
    functions::check_arity_min(arity_min, arg_count)?;
    functions::check_arg_types(arg_types, arg_start, vm)?;
    check_arg_pairs(arity_min, arg_count)?;

    let tbl_name = stack[arg_start].to_ascii_lower(vm);
    let table = Table::new(&tbl_name)?;

    let result_col_hash = if sel_context == Select::Row {
        let (name, hash) = stack[arg_start + 1].to_string_hash(vm);
        let _ = &table.col_position(&name)?; // error if not found

        hash
    } else {
        0 // never used
    };

    let result_limit = match sel_context {
        Select::Row => 1,
        Select::Rowset | Select::RowsetCs => RESULT_MAX,
        Select::Ordered | Select::OrderedCs => {
            let limit = f64::trunc(stack[arg_start + 1].to_num(vm)) as usize;
            if !(1..=RESULT_MAX).contains(&limit) {
                RESULT_MAX
            } else {
                limit
            }
        }
    };

    let filter_col_offset = match sel_context {
        Select::Row => arg_start + 2,
        Select::Rowset | Select::RowsetCs => arg_start + 1,
        Select::Ordered | Select::OrderedCs => arg_start + 3,
    };
    let filter_col = stack[filter_col_offset].to_string(vm);

    let collation = if sel_context == Select::RowsetCs || sel_context == Select::OrderedCs {
        String::with_capacity(0)
    } else {
        String::from(NO_CASE)
    };

    let order = if sel_context == Select::Ordered || sel_context == Select::OrderedCs {
        let result_order = stack[arg_start + 2].to_string(vm);
        let result_order_str = result_order.trim();
        let order_parts = result_order_str
            .split_ascii_whitespace()
            .collect::<Vec<_>>();

        if order_parts.len() != 2 {
            return Err(Value::error(
                "Argument for sorting must include a column name and direction \
                    (e.g., `my_column ASC` or `my_column DESC`)."
                    .to_owned(),
            ));
        }
        let order_col = order_parts[0];
        let order_dir = order_parts[1];
        let _ = &table.col_position(order_col)?; // error if not found

        if !("asc".eq_ignore_ascii_case(order_dir) || "desc".eq_ignore_ascii_case(order_dir)) {
            return Err(Value::error(format!(
                "Unexpected sorting direction — got `{order_dir}`, but expected `ASC` or `DESC` (case-insensitive)."
            )));
        }

        format!("\nORDER BY {order_col} {order_dir}")
    } else {
        String::with_capacity(0)
    };

    // table and column names are validated prior to prepared statement creation
    let mut sql = format!(
        r"
WITH _types AS (SELECT lower(group_concat(type)) _types FROM pragma_table_info('{tbl_name}'))
SELECT * FROM {tbl_name},_types
WHERE {filter_col} = ?1 {collation}"
    );

    if arg_count > arity_min {
        let ord_start = 2;
        let start = arg_start + arity_min as usize;

        db::append_sql_args(
            AppendType::Predicate,
            &mut sql,
            ord_start,
            collation,
            (start, stack.len()),
            vm,
        )?;
    }
    sql += &order;
    sql += &format!("\nLIMIT {result_limit}");

    let mut param_data: Vec<Box<dyn ToSql>> = Vec::new();
    let param_count = stack.len() - filter_col_offset;

    let stmt = db::prep_stmt(
        &table,
        &sql,
        &mut param_data,
        (filter_col_offset, param_count),
        vm,
    )?;

    let params = param_data
        .iter()
        .map(|val| val.as_ref())
        .collect::<Vec<_>>();
    let rows = db::exec_stmt(stmt, params.as_slice(), vm)?;

    let val = if sel_context == Select::Row {
        let mut result_col_val = Value::null();

        if !rows.is_empty() {
            let row = rows[0].to_row();

            if let Some(val) = row.map.get(&result_col_hash) {
                if !val.is_null() {
                    result_col_val = (*val).clone();
                }
            }
        }

        result_col_val
    } else {
        Value::rowset(rows)
    };

    Ok(val)
}

fn exec_update(
    upd_context: Update,
    arity_min: u8,
    arg_types: &[ArgType],
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    const FILTER_COL_MAX: usize = 125;
    let stack = &vm.stack;
    functions::check_arity_min(arity_min, arg_count)?;
    functions::check_arg_types(arg_types, arg_start, vm)?;
    check_arg_pairs(arity_min, arg_count)?;

    let is_upsert = upd_context == Update::UpsertSend || upd_context == Update::UpsertReq;

    let tbl_name = stack[arg_start].to_ascii_lower(vm);
    let table = Table::new(&tbl_name)?;

    let filter_col_offset = arg_start + 2;
    let filter_col_count = f64::trunc(stack[filter_col_offset - 1].to_num(vm)) as usize;
    if !(1..=FILTER_COL_MAX).contains(&filter_col_count) {
        return Err(Value::error(format!(
            "Unexpected number specified to build the `WHERE` clause — got {filter_col_count}, but expected between 1 and {FILTER_COL_MAX}."
        )));
    }

    let total_col_count = (stack.len() - filter_col_offset) / 2;
    if filter_col_count == total_col_count {
        return Err(Value::error(format!(
            "The number {filter_col_count} was specified to build the `WHERE` clause, but only {total_col_count} name-value pairs in total
were passed to the function. At least one additional name-value pair (a column to receive an updated value)
is required."
        )));
    }
    if filter_col_count > total_col_count {
        return Err(Value::error(format!(
            "The number {filter_col_count} was specified to build the `WHERE` clause, but only {total_col_count} name-value pairs in total
were passed to the function."
        )));
    }
    let filter_col = stack[filter_col_offset].to_string(vm);

    let update_col_offset = filter_col_offset + (filter_col_count * 2);
    let update_col_count = (stack.len() - update_col_offset) / 2;
    let update_col = stack[update_col_offset].to_string(vm);

    let collation = String::from(NO_CASE);

    // table and column names are validated prior to prepared statement creation

    let mut update_args = if is_upsert {
        format!("{update_col} = excluded.{update_col}")
    } else {
        format!("SET {update_col} = ?1")
    };
    if update_col_count > 1 {
        let append_type = if is_upsert {
            AppendType::Upsert
        } else {
            AppendType::Update
        };
        let collation = String::with_capacity(0);
        let ord_start = 2;
        let start = update_col_offset + 2;

        db::append_sql_args(
            append_type,
            &mut update_args,
            ord_start,
            collation,
            (start, stack.len()),
            vm,
        )?;
    }

    let filter_ord = if is_upsert { 1 } else { update_col_count + 1 };
    let mut filter_args = format!("WHERE {filter_col} = ?{filter_ord} {collation}");
    if filter_col_count > 1 {
        let start = filter_col_offset + 2;

        db::append_sql_args(
            AppendType::Predicate,
            &mut filter_args,
            filter_ord + 1,
            collation,
            (start, update_col_offset),
            vm,
        )?;
    }

    let mut sql = if is_upsert {
        format!(
            r"
INSERT INTO {tbl_name} ("
        )
    } else {
        format!(
            r"
UPDATE {tbl_name}
{update_args}
{filter_args}"
        )
    };

    if is_upsert {
        let collation = String::with_capacity(0); // unused
        let ord_start = 1;
        let start = filter_col_offset;

        db::append_sql_args(
            AppendType::Identifier,
            &mut sql,
            ord_start,
            collation,
            (start, stack.len()),
            vm,
        )?;
        sql += ") VALUES (";

        let mut ord = 1;
        let mut i = 0;
        while i < filter_col_count + update_col_count {
            if i > 0 {
                sql += ",";
            }
            sql += &format!("?{ord}");
            ord += 1;
            i += 1;
        }
        sql += ")";
        sql += &format!(
            r"
ON CONFLICT DO UPDATE SET
    {update_args}
{filter_args}"
        );
    }

    let mut param_data: Vec<Box<dyn ToSql>> = Vec::new();
    let param_count = stack.len() - filter_col_offset;

    let mut stmt = db::prep_stmt(
        &table,
        &sql,
        &mut param_data,
        (filter_col_offset, param_count),
        vm,
    )?;

    let params = if is_upsert {
        param_data
            .iter()
            .map(|val| val.as_ref())
            .collect::<Vec<_>>()
    } else {
        let mut params: Vec<&dyn ToSql> = Vec::new();

        // function args (filter..update) and SQL statement params (update..filter) are inverted;
        // `prep_stmt` parameterizes according to function args, so transposition is required
        for param in &param_data[filter_col_count..param_data.len()] {
            params.push(param.as_ref()); // push update args first
        }
        for param in &param_data[..filter_col_count] {
            params.push(param.as_ref()); // then filter args after
        }

        params
    };

    let rows_affected = stmt.execute(params.as_slice())?;

    match upd_context {
        Update::Send => Ok(Value::null()),
        Update::Req | Update::UpsertSend | Update::UpsertReq => {
            Ok(Value::num(rows_affected as f64))
        }
    }
}

fn exec_delete(
    del_context: Delete,
    arity_min: u8,
    arg_types: &[ArgType],
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    let stack = &vm.stack;
    functions::check_arity_min(arity_min, arg_count)?;
    functions::check_arg_types(arg_types, arg_start, vm)?;
    check_arg_pairs(arity_min, arg_count)?;

    let tbl_name = stack[arg_start].to_ascii_lower(vm);
    let table = Table::new(&tbl_name)?;

    let filter_col_offset = arg_start + 1;
    let filter_col = stack[filter_col_offset].to_string(vm);

    let collation = String::from(NO_CASE);

    // table and column names are validated prior to prepared statement creation
    let mut sql = format!(
        r"
DELETE FROM {tbl_name}
WHERE {filter_col} = ?1 {collation}"
    );

    if arg_count > arity_min {
        let ord_start = 2;
        let start = arg_start + arity_min as usize;

        db::append_sql_args(
            AppendType::Predicate,
            &mut sql,
            ord_start,
            collation,
            (start, stack.len()),
            vm,
        )?;
    }

    let mut param_data: Vec<Box<dyn ToSql>> = Vec::new();
    let param_count = stack.len() - filter_col_offset;

    let mut stmt = db::prep_stmt(
        &table,
        &sql,
        &mut param_data,
        (filter_col_offset, param_count),
        vm,
    )?;

    let params = param_data
        .iter()
        .map(|val| val.as_ref())
        .collect::<Vec<_>>();
    let rows_affected = stmt.execute(params.as_slice())?;

    match del_context {
        Delete::Send => Ok(Value::null()),
        Delete::Req => Ok(Value::num(rows_affected as f64)),
    }
}
