
mod vm {
    #[derive(Debug, Clone)]
    pub struct Vm(
        std::collections::HashMap<Ident, Value>,
        HashMap<String, fn(Vec<Value>) -> Value>,
    );

    impl Vm {
        pub fn new() -> Self {
            let mut map_builtins = HashMap::new();
            map_builtins.insert("print".to_string(), Vm::print as fn(Vec<Value>) -> Value);
            map_builtins.insert("println".to_string(), Vm::println);
            Vm(HashMap::new(), map_builtins)
        }
        
        pub fn eval_expr(&mut self, expr: Expr) -> Result<Value, Error> {
            match expr {
                Expr::Empty => Ok(Value::None),
                Expr::Block { body } => {
                    let mut last = Value::None;
                    for expr in body {
                        last = self.eval_expr(expr)?;
                    }
                    Ok(last)
                }
                Expr::Literal { value } => Ok(match value {
                    Literal::Number(n) => Value::Number(n),
                    Literal::String(s) => Value::String(s),
                    Literal::Bool(b) => Value::Bool(b),
                }),
                Expr::Ident { ref ident } => match self.get_ident(Ident(ident.clone())) {
                    Some(value) => Ok(value.clone()),
                    None => Err(Error::VarNotFound(VarNotFoundError {
                        var_name: ident.clone(),
                    })),
                },
                Expr::BinOp { op, left, right } => {
                    let left = self.eval_expr(*left)?;

                    let right = self.eval_expr(*right)?;
                    Ok(match op {
                        Op::Add => left.add(&right)?,
                        Op::Sub => left.sub(&right)?,
                        Op::Mul => left.mul(&right)?,
                        Op::Div => left.div(&right)?,
                        Op::Mod => left.modulo(&right)?,
                        Op::Eq => left.eq(&right)?,
                        Op::Neq => left.neq(&right)?,
                        Op::Gt => left.gt(&right)?,
                        Op::Lt => left.lt(&right)?,
                        Op::Ge => left.ge(&right)?,
                        Op::Le => left.le(&right)?,
                        Op::And => left.and(&right)?,
                        Op::Or => left.or(&right)?,
                    })
                }
                Expr::IfThen { cond, then } => {
                    let v = self.eval_expr(*cond)?;
                    if let Value::Bool(c) = v {
                        if c {
                            Ok(self.eval_expr(*then)?)
                        } else {
                            Ok(Value::None)
                        }
                    } else {
                        Err(Error::TypeMismatch(TypeMismatchError {
                            expected: "bool".to_string(),
                            found: v.get_type(),
                        }))
                    }
                }
                Expr::IfThenElse { cond, then, else_ } => {
                    let v = self.eval_expr(*cond)?;
                    if let Value::Bool(n) = v {
                        if n {
                            self.eval_expr(*then)
                        } else {
                            self.eval_expr(*else_)
                        }
                    } else {
                        Err(Error::TypeMismatch(TypeMismatchError {
                            expected: "bool".to_string(),
                            found: v.get_type(),
                        }))
                    }
                }
                Expr::Assign { name, value } => {
                    let value_evaluate = self.eval_expr(*value)?;
                    if self.get_ident(Ident(name.clone())).is_some() {
                        return Err(Error::VarAlreadyDefined(VarAlreadyDefinedError {
                            var_name: name,
                        }));
                    }
                    self.set_ident(Ident(name), value_evaluate.clone());
                    Ok(Value::None)
                }
                Expr::While { ref cond, ref body } => {
                    while self.eval_expr(*cond.clone())? == Value::Bool(true) {
                        self.eval_expr(*body.clone())?;
                    }
                    Ok(Value::None)
                }
                Expr::For {
                    ref name,
                    ref iter,
                    ref body,
                } => {
                    let name_str = match *name.clone() {
                        Expr::Ident { ident } => ident,
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "string".to_string(),
                                found: "ident".to_string(),
                            }))
                        }
                    };

                    let iter = self.eval_expr(*iter.clone())?;
                    match iter {
                        Value::List(ref l) => {
                            let mut last = Value::None;
                            for item in l {
                                self.set_ident(Ident(name_str.clone()), item.clone());
                                last = self.eval_expr(*body.clone())?;
                            }
                            Ok(last)
                        }
                        Value::Range(r) => {
                            let mut last = Value::None;
                            for i in r {
                                self.set_ident(Ident(name_str.clone()), Value::Number(i as f64));
                                last = self.eval_expr(*body.clone())?;
                            }
                            Ok(last)
                        }
                        _ => Err(Error::TypeMismatch(TypeMismatchError {
                            expected: "list".to_string(),
                            found: iter.get_type(),
                        })),
                    }
                }
                Expr::FunDef {
                    ref name,
                    ref args,
                    ref body,
                } => {
                    if self.1.clone().into_iter().any(|x| x.0 == name.clone()) {
                        return Err(Error::IsBuiltin(IsBuiltinError { name: name.clone() }));
                    }
                    let mut args_vec = Vec::new();
                    for arg in args {
                        let arg_name = match arg {
                            Expr::Ident { ref ident } => ident.clone(),
                            _ => {
                                return Err(Error::TypeMismatch(TypeMismatchError {
                                    expected: "ident".to_string(),
                                    found: "unknown".to_string(),
                                }))
                            }
                        };
                        args_vec.push(Ident(arg_name));
                    }
                    self.set_ident(
                        Ident(name.clone()),
                        Value::Function {
                            name: name.clone(),
                            args: args_vec.clone(),
                            body: *body.clone(),
                        },
                    );
                    Ok(Value::Function {
                        name: name.clone(),
                        args: args_vec.clone(),
                        body: *body.clone(),
                    })
                }
                Expr::Call {
                    ref name, ref args, ..
                } => {
                    let cparg = args.clone();
                    let b = self.1.clone();
                    if b.clone().into_iter().any(|x| x.0 == name.clone()) {
                        let v = b.get(name).unwrap().clone();
                        let args_iter = args.iter();

                        let args_map = args_iter.map(|x| -> Value {
                            match self.eval_expr(x.clone()) {
                                Ok(v) => v,
                                Err(_e) => Value::None,
                            }
                        });

                        let args = args_map.collect::<Vec<Value>>();
                        return Ok(v(args));
                    }
                    let mut new_vm = Vm::new();
                    let copy_self = self.clone();
                    match copy_self.get_ident(Ident(name.clone())) {
                        Some(f) => match f {
                            &Value::Function {
                                ref name,
                                ref args,
                                ref body,
                            } => {
                                new_vm.set_ident(Ident(name.clone()), f.clone());
                                for (i, arg) in args.iter().enumerate() {
                                    let ev_arg = self.eval_expr(cparg[i].clone())?;
                                    new_vm.set_ident(arg.clone(), ev_arg.clone());
                                }
                                new_vm.eval_expr(body.clone())
                            }
                            _ => Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "function".to_string(),
                                found: f.get_type(),
                            })),
                        },
                        None => Err(Error::FunctionNotFound(FunctionNotFoundError {
                            name: name.clone(),
                        })),
                    }
                }
                Expr::List { ref elems } => {
                    let mut list = Vec::new();
                    for elem in elems {
                        list.push(self.eval_expr(elem.clone())?);
                    }
                    Ok(Value::List(list))
                }
                Expr::Index {
                    ref name,
                    ref index,
                } => {
                    let real_name = match **name {
                        Expr::Ident { ref ident } => ident.clone(),
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "ident".to_string(),
                                found: "unknown".to_string(),
                            }))
                        }
                    };
                    let copy_vm = self.clone();
                    let list = match copy_vm.get_ident(Ident(real_name.clone())) {
                        Some(Value::List(list)) => list,
                        None => {
                            return Err(Error::VarNotFound(VarNotFoundError {
                                var_name: real_name,
                            }))
                        }
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "list".to_string(),
                                found: self.get_ident(Ident(real_name)).unwrap().get_type(),
                            }))
                        }
                    };

                    let index = self.eval_expr(*index.clone())?;
                    match index {
                        Value::Number(num) => {
                            if num < 0.0 {
                                return Err(Error::IndexOutOfBounds(IndexOutOfBoundsError {
                                    index: num as i32,
                                    name: real_name,
                                }));
                            }
                            if num as usize >= list.len() {
                                return Err(Error::IndexOutOfBounds(IndexOutOfBoundsError {
                                    index: num as i32,
                                    name: real_name,
                                }));
                            }
                            Ok(list[num as usize].clone())
                        }
                        Value::Range(r) => {
                            if r.start >= list.len() as isize {
                                return Err(Error::IndexOutOfBounds(IndexOutOfBoundsError {
                                    index: r.start as i32,
                                    name: real_name,
                                }));
                            }

                            if r.end > list.len() as isize {
                                return Err(Error::IndexOutOfBounds(IndexOutOfBoundsError {
                                    index: r.end as i32,
                                    name: real_name,
                                }));
                            }

                            Ok(Value::List(list[r.start as usize..r.end as usize].to_vec()))
                        }
                        _ => Err(Error::TypeMismatch(TypeMismatchError {
                            expected: "number".to_string(),
                            found: index.get_type(),
                        })),
                    }
                }
                Expr::Range { ref start, ref end } => {
                    let start = self.eval_expr(*start.clone())?;
                    let end = self.eval_expr(*end.clone())?;
                    let start = match start {
                        Value::Number(n) => n,
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "number".to_string(),
                                found: start.get_type(),
                            }))
                        }
                    };
                    let end = match end {
                        Value::Number(n) => n,
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "number".to_string(),
                                found: end.get_type(),
                            }))
                        }
                    };

                    Ok(Value::Range(start as isize..end as isize))
                },
                Expr::StructDef {
                    ref name,
                    ref fields,
                } => {
                    let mut f = Vec::new();
                    
                    for field in fields {
                        match field {
                            Expr::Ident { ref ident } => f.push(ident.clone()),
                            _ => {
                                return Err(Error::TypeMismatch(TypeMismatchError {
                                    expected: "ident".to_string(),
                                    found: "unknown".to_string(),
                                }))
                            } 
                        }
                        
                    }
                    let mut nf = Vec::new();
                    for field in fields {
                        nf.push(match field {
                            Expr::Ident { ident } => Ident(ident.clone()),
                            _ => {
                                return Err(Error::TypeMismatch(TypeMismatchError {
                                    expected: "ident".to_string(),
                                    found: "unknown".to_string(),
                                }))
                            }
                        });
                    }
                    self.set_ident(Ident(name.clone()), Value::DefStruct {
                        name: name.clone(),
                        fields: nf,
                        function: HashMap::new()
                    });
                    Ok(Value::None)
                },
                Expr::CallStruct { ref name, ref args } => {
                    let copy_self = self.clone();
                    match copy_self.get_ident(Ident(name.clone())) {
                        Some(f) => match f {
                            &Value::DefStruct {
                                ref fields,
                                ..
                            } => {
                                let mut map = HashMap::new();
                                let mut a ;
                                let mut _v;
                                for (arg, value) in args {
                                    a = match arg {
                                        Expr::Ident { ref ident } => ident.clone(),
                                        _ => {
                                            return Err(Error::TypeMismatch(TypeMismatchError {
                                                expected: "ident".to_string(),
                                                found: "unknown".to_string(),
                                            }))
                                        }
                                    };
                                    _v = self.eval_expr(value.clone())?;
                                    for field in fields {
                                        let Ident(f) = field.clone();
                                        if f == a {
                                            map.insert(field.clone(), self.eval_expr(value.clone())?);
                                        }

                                    }
                                }
                                Ok(Value::CallStruct {
                                    name: name.clone(),
                                    fields: map,
                                })
                            }
                            _ => Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "struct".to_string(),
                                found: f.get_type(),
                            })),
                        },
                        None => Err(Error::StructNotFound(StructNotFoundError {
                            name: name.clone(),
                        })),
                    }
                },
                Expr::GetAttr { name , attr } => {
                    match self.get_ident(Ident(name)) {
                        Some(Value::CallStruct { ref fields , ..}) => {
                            match fields.get(&Ident(attr.clone())) {
                                Some(v) => return Ok(v.clone()),
                                None => {
                                    return Err(Error::AttrNotFound(AttrNotFoundError {
                                        attr_name: attr
                                    }))
                                }
                            }
                        }
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "struct".to_string(),
                                found: "unknown".to_string(),
                            }))
                        }
                    };
                },
                Expr::Impl { ref name_struct , ref name_method, args, body } => {
                    let fiw;
                    let mut fuw;
                    match self.get_ident(Ident(name_struct.clone())) {
                        Some(Value::DefStruct { ref fields, ref function , ..}) => {
                            fiw = fields.clone();
                            fuw = function.clone();
                        },
                        None => {
                            return Err(Error::StructNotFound(StructNotFoundError {
                                name: name_struct.clone(),
                            }))
                        }
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "struct".to_string(),
                                found: "unknown".to_string(),
                            }))
                        }
                    };

                    let mut args_vec = Vec::new();
                    for arg in args {
                        args_vec.push(match arg {
                            Expr::Ident { ref ident } => Ident(ident.clone()),
                            _ => {
                                return Err(Error::TypeMismatch(TypeMismatchError {
                                    expected: "ident".to_string(),
                                    found: "unknown".to_string(),
                                }))
                            }
                        });
                    }
                    let f = Value::Function { name: name_method.clone(), args: args_vec, body: *body };
                    fuw.insert(name_method.clone(), f);
                    self.set_ident(Ident(name_struct.clone()), Value::DefStruct { name: name_struct.clone(), fields: fiw, function: fuw });
                    Ok(Value::None)
                },
                Expr::GetFunc { name , func , args } => {
                    let call_struct;
                    let field_struct;
                    let s = match self.get_ident(Ident(name)) {
                        Some(Value::CallStruct { name: n, fields: fi }) => {
                            call_struct = Value::CallStruct { name: n.clone(), fields: fi.clone() };
                            match &self.get_ident(Ident(n.clone())) {
                                Some(Value::DefStruct { fields: f, function: fu , ..}) => {
                                    field_struct = f;
                                    match fu.get(&func) {
                                        Some(v) => v.clone(),
                                        None => {
                                            return Err(Error::FunctionNotFound(FunctionNotFoundError {
                                                name: func
                                            }))
                                        }
                                    }
                                }
                                _ => {
                                    return Err(Error::TypeMismatch(TypeMismatchError {
                                        expected: "struct".to_string(),
                                        found: "unknown".to_string(),
                                    }))
                                }
                            }
                        }
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "struct".to_string(),
                                found: "unknown".to_string(),
                            }))
                        }
                    };

                    let mut args_vec = Vec::new();
                    let mut new_vm = Vm::new();
                    for arg in args {
                        args_vec.push(self.clone().eval_expr(arg.clone())?);
                    }
                    new_vm.set_ident(Ident("self".to_string()), call_struct);

                    for (value, arg) in args_vec.iter().zip(field_struct.iter()) {
                        new_vm.set_ident(arg.clone(), value.clone());
                    }
                    
                    new_vm.eval_expr(match s { 
                        Value::Function {body , ..} => {
                            body
                        }
                        _ => {
                            return Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "function".to_string(),
                                found: "unknown".to_string(),
                            }))
                        }
                    })
                    
                    
                },
                Expr::SetVar { name, value } => {
                    if let None = self.get_ident(Ident(name.clone())) {
                        return Err(Error::VarNotFound(VarNotFoundError {
                            var_name: name.clone(),
                        }));
                    }
                    let v = self.eval_expr(*value.clone())?;
                    self.set_ident(Ident(name), v);
                    Ok(Value::None)
                },
                Expr::IOp { op, name, value } => {
                    let v = self.eval_expr(*value.clone())?;
                    match op {
                        IOp::IAdd => self.iadd(name, v),
                        IOp::ISub => self.isub(name, v),
                        IOp::IMul => self.imul(name, v),
                        IOp::IDiv => self.idiv(name, v)
                    }
                },
                Expr::Match { value, cases } => {
                    let mut return_value = Value::None;
                    for i in cases {
                        let _case = self.eval_expr(i.0);
                        match self.eval_expr(*value.clone())?.clone() {
                            _case => {
                                let mut new_vm = Vm::new();
                                return_value = new_vm.eval_expr(i.1)?;
                            }
                        }
                    }
                    Ok(return_value)
                }
                
            }
        }

        pub fn set_ident(&mut self, ident: Ident, value: Value) {
            self.0.insert(ident, value);
        }

        pub fn get_ident(&self, ident: Ident) -> Option<&Value> {
            self.0.get(&ident)
        }

        pub fn iadd(&mut self, a: String, b: Value) -> Result<Value, Error> {
            match b {
                Value::Number(b) => {
                    if self.exists(Ident(a.clone())) {
                        let v = self.get_ident(Ident(a.clone())).unwrap().clone();
                        match v {
                            Value::Number(v) => {
                                self.set_ident(Ident(a), Value::Number(v + b));
                                Ok(Value::None)
                            }
                            _ => Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "number".to_string(),
                                found: v.get_type(),
                            })),
                        }
                    } else {
                        return Err(Error::VarNotFound(VarNotFoundError {
                            var_name: a,
                        }));
                    }
                },
                _ => Err(Error::TypeMismatch(TypeMismatchError {
                    expected: "int or float".to_string(),
                    found: "unknown".to_string(),
                })),
            }
        }

        pub fn isub(&mut self, a: String, b: Value) -> Result<Value, Error> {
            match b {
                Value::Number(b) => {
                    if self.exists(Ident(a.clone())) {
                        let v = self.get_ident(Ident(a.clone())).unwrap().clone();
                        match v {
                            Value::Number(v) => {
                                self.set_ident(Ident(a), Value::Number(v - b));
                                Ok(Value::None)
                            }
                            _ => Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "number".to_string(),
                                found: v.get_type(),
                            })),
                        }
                    } else {
                        return Err(Error::VarNotFound(VarNotFoundError {
                            var_name: a,
                        }));
                    }
                },
                _ => Err(Error::TypeMismatch(TypeMismatchError {
                    expected: "int or float".to_string(),
                    found: "unknown".to_string(),
                })),
            }
        }

        pub fn imul(&mut self, a: String, b: Value) -> Result<Value, Error> {
            match b {
                Value::Number(b) => {
                    if self.exists(Ident(a.clone())) {
                        let v = self.get_ident(Ident(a.clone())).unwrap().clone();
                        match v {
                            Value::Number(v) => {
                                self.set_ident(Ident(a), Value::Number(v * b));
                                Ok(Value::None)
                            }
                            _ => Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "number".to_string(),
                                found: v.get_type(),
                            })),
                        }
                    } else {
                        return Err(Error::VarNotFound(VarNotFoundError {
                            var_name: a,
                        }));
                    }
                },
                _ => Err(Error::TypeMismatch(TypeMismatchError {
                    expected: "int or float".to_string(),
                    found: "unknown".to_string(),
                })),
            }
        }

        pub fn idiv(&mut self, a: String, b: Value) -> Result<Value, Error> {
            match b {
                Value::Number(b) => {
                    if self.exists(Ident(a.clone())) {
                        let v = self.get_ident(Ident(a.clone())).unwrap().clone();
                        match v {
                            Value::Number(v) => {
                                self.set_ident(Ident(a), Value::Number(v / b));
                                Ok(Value::None)
                            }
                            _ => Err(Error::TypeMismatch(TypeMismatchError {
                                expected: "number".to_string(),
                                found: v.get_type(),
                            })),
                        }
                    } else {
                        return Err(Error::VarNotFound(VarNotFoundError {
                            var_name: a,
                        }));
                    }
                },
                _ => Err(Error::TypeMismatch(TypeMismatchError {
                    expected: "int or float".to_string(),
                    found: "unknown".to_string(),
                })),
            }
        }



        pub fn exists(&self, ident: Ident) -> bool {
            self.0.contains_key(&ident)
        }

        pub fn print(args: Vec<Value>) -> Value {
            for i in args {
                print!("{}", i.display_value());
            }
            Value::None
        }

        pub fn println(args: Vec<Value>) -> Value {
            for i in args {
                print!("{}", i.display_value());
            }
            println!();
            Value::None
        }
    }
}