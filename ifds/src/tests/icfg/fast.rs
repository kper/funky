use crate::icfg::tabulation::fast::TabulationFast;
use crate::icfg::tikz::render_to;
use crate::solver::Request;
use insta::assert_snapshot;
use log::error;

use crate::grammar::*;

use crate::icfg::flowfuncs::taint::flow::TaintNormalFlowFunction;
use crate::icfg::flowfuncs::taint::initial::TaintInitialFlowFunction;

use std::fs::{create_dir, OpenOptions};
use std::io::Write;

/// Write the IR to a seperate file. This makes it possible
/// to run it in the UI.
fn write_ir(name: &str, ir: &str) {
    let _ = create_dir("src/tests/icfg/ir_code");
    let mut fs = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("src/tests/icfg/ir_code/{}.ir", name))
        .unwrap();
    fs.write_all(&ir.as_bytes()).unwrap();
}

macro_rules! ir {
    ($name:expr, $req:expr, $ir:expr) => {
        let mut convert = TabulationFast::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let res = convert.visit(&prog, &$req);

        if let Err(err) = res {
            error!("ERROR: {}", err);
            err.chain()
                .skip(1)
                .for_each(|cause| error!("because: {}", cause));
            panic!("")
        }

        write_ir($name, $ir);

        let (graph, state) = res.unwrap();

        let output = render_to(&graph, &state);

        assert_snapshot!(format!("{}_dot", $name), output);
    };
}

#[test]
fn test_ir_const() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };

    ir!(
        "test_ir_const",
        req,
        "
         define test (result 0) (define %0) {
            %0 = 1
         };
    "
    );
}

#[test]
fn test_ir_simple_store() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };

    ir!(
        "test_ir_simple_store",
        req,
        "
         define test (param %0) (result 0) (define %0 %1) {
            STORE FROM %0 OFFSET 0 + %0 ALIGN 2 32
         };
    "
    );
}

#[test]
fn test_ir_simple_load() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };

    ir!(
        "test_ir_simple_load",
        req,
        "
         define test (param %0) (result 0) (define %0 %1) {
            %1 = LOAD OFFSET 0 + %0 ALIGN 0
         };
    "
    );
}

#[test]
fn test_ir_double_assign() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_double_const",
        req,
        "
         define test (result 0) (define %0 %1 %2){
            %0 = 1
            %1 = 1
            %2 = %0
            %2 = %1
         };
    "
    );
}

#[test]
fn test_ir_double_assign_with_params() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_double_const_with_params",
        req,
        "
         define test (param %0) (result 0) (define %0 %1 %2){
            %1 = 1
            %2 = %0
            %2 = %1
         };
    "
    );
}

#[test]
fn test_ir_chain_assign() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_chain_assign",
        req,
        "
         define test (result 0) (define %0 %1 %2 %3){
            %0 = 1
            %1 = 1
            %2 = %0
            %3 = %2
         };
    "
    );
}

#[test]
fn test_ir_unop() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_unop",
        req,
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 = op %0
            %1 = op %0   
        };"
    );
}

#[test]
fn test_ir_binop() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_binop",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 1
            %2 = %0 op %1
            %2 = %1 op %0   
        };"
    );
}

#[test]
fn test_ir_binop_offset() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_binop_offset",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 1
            %2 = %0 op %1
            %2 = %1 op %0   
        };"
    );
}

#[test]
fn test_ir_phi() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_phi",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 1
            %2 = phi %0 %1
        };"
    );
}

#[test]
fn test_ir_killing_op() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_killing_op",
        req,
        "define test (result 0) (define %0 %1 %2)  {
            %0 = 1
            %1 = 1
            KILL %0
            KILL %1
            %2 = 1
        };"
    );
}

#[test]
fn test_ir_block() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_block",
        req,
        "define test (result 0) (define %0 %1) {
            BLOCK 0
            %0 = 1
            GOTO 1
            BLOCK 1
            %1 = 2
        };"
    );
}

#[test]
fn test_ir_if_else() {
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_if_else",
        req,
        "define main (result 0) (define %0 %1 %2) {
            BLOCK 0
            %0 = 1
            IF %1 THEN GOTO 1 ELSE GOTO 2 
            BLOCK 1
            %1 = 2
            %2 = 3
            GOTO 3
            BLOCK 2
            %2 = 4
            GOTO 3
            BLOCK 3
            %0 = %2
        };
        "
    );
}

#[test]
fn test_ir_if() {
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_if",
        req,
        "define main (result 0) (define %0 %1 %2) {
            BLOCK 0
            %0 = 1
            IF %1 THEN GOTO 0
            %1 = 2
            %2 = 3
        };
        "
    );
}

#[test]
fn test_ir_loop() {
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_loop",
        req,
        "define main (result 0) (define %0 %1) {
            BLOCK 0
            %0 = 1
            %1 = 2
            GOTO 0 
        };
        "
    );
}

#[test]
fn test_ir_table() {
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_table",
        req,
        "define main (result 0) (define %0 %1 %2) {
            BLOCK 0
            %0 = 1
            %1 = 2
            %2 = 3
            BLOCK 1
            %1 = 2
            %2 = 3
            BLOCK 2
            %2 = 4
            TABLE GOTO 0 1 2 ELSE GOTO 2
        };
        "
    );
}

#[test]
fn test_ir_functions() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_functions",
        req,
        "define test (result 0) (define %0) {
            %0 = 1
            CALL mytest(%0)
        };
        define mytest (param %0) (result 0) (define %0 %1)  {
            %0 = 2   
            %1 = 3
            RETURN;
        };"
    );
}

#[test]
fn test_ir_multiple_functions() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_multiple_functions",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 <- CALL mytest(%0)
            %2 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0)  {
            RETURN %0;
        };"
    );
}

#[test]
fn test_ir_functions_rename_reg() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_functions_rename_regs",
        req,
        "define test (result 0) (define %0) {
            %0 = 1
            %0 <- CALL mytest(%0)
        };
        define mytest (param %5) (result 1) (define %5 %6)  {
            %6 = %5
            RETURN %6;
        };"
    );
}

#[test]
fn test_ir_return_values() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_values",
        req,
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %1;
        };"
    );
}

#[test]
fn test_ir_return_passed_value() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_passed_value",
        req,
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            RETURN %0;
        };"
    );
}

#[test]
fn test_ir_return_values2() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_values2",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 2
            %2 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %1 = 3
            RETURN %0;
        };"
    );
}

#[test]
fn test_ir_return_values3() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_values3",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 2
            %2 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            RETURN %0;
        };"
    );
}

#[test]
fn test_ir_overwrite_return_values() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_overwrite_return_values",
        req,
        "define test (result 0) (define %0) {
            %0 = 1
            %0 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %1;
        };"
    );
}

#[test]
fn test_ir_early_return() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_early_return",
        req,
        "define test (result 0) (define %0 %1 %2 %3) {
            %0 = 1
            %1 <- CALL mytest(%0)
            %2 <- CALL mytestfoo(%0)
            %3 <- CALL mytestfoo(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %1;
        };
        define mytestfoo (param %0) (result 1) (define %0 %1) {
            %1 = 3
            RETURN %0;
        };
        "
    );
}

#[test]
fn test_ir_return_double() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return",
        req,
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %0 %1 <- CALL mytest(%0)
            %1 = 2
        };
        define mytest (param %0) (result 2) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %0 %1;
        };
        "
    );
}

#[test]
fn test_ir_return_branches() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_branches",
        req,
        "define test (result 0) (define %0 %1) {
            %0 = 5
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1 %2) {
            %1 = 1
            IF %1 THEN GOTO 1 ELSE GOTO 2 
            BLOCK 1
            %1 = 2
            %2 = 3
            RETURN %1;
            GOTO 3
            BLOCK 2
            %2 = 4
            GOTO 3
            BLOCK 3
            RETURN %0;
        };
        "
    );
}

#[test]
fn test_ir_self_loop() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_self_loop",
        req,
        "define test (param %0) (result 1) (define %0 %1 %2) {
            %2 = 1
            %0 = 5
            %1 <- CALL test(%0)
            %0 = %1
            RETURN %0;
        };
        "
    );
}

#[test]
fn test_global_get_and_set() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_global_get_and_set",
        req,
        "
        define 0 (param %0) (result 0) (define %-2 %-1 %0 %1) {
        %1 = %-1
        %-2 = %1
        };
    "
    );
}

#[test]
fn test_global_set() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_global_set",
        req,
        "
        define 0 (param %0) (result 0) (define %-1 %0 %1) {
            %0 = 1
            %-1 = %0
            %1 <- CALL 1()
        };

        define 1 (param) (result 1) (define %-1 %0) {
            %0 = %-1
            RETURN %0;
        };
    "
    );
}

#[test]
fn test_global_get_and_set_multiple_functions() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_global_get_and_set_multiple_functions",
        req,
        "
        define 0 (param %0) (result 0) (define %-2 %-1 %0 %1 %2) {
        %1 = %-1
        %-2 = %1
        %2 = 1
        %0 <- CALL 1 (%2)
        %1 <- CALL 2 ()
        };

        define 1 (param %0) (result 1) (define %-2 %0) {
        %0 = %-2
        RETURN %0;
        };

        define 2 (result 1) (define %-2 %0) {
        %0 = 1
        %-2 = 1
        RETURN %0;
        };
    "
    );
}

#[test]
fn test_global_call() {
    let req = Request {
        variable: None,
        function: "1".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_global_call",
        req,
        "
        define 0 (param %0) (result 0) (define %-2 %-1 %0 %1) {
        BLOCK 0
        %1 = %0
        %-1 = %1
        RETURN ;
        };
        define 1 (param %0) (result 0) (define %-2 %-1 %0 %1 %2) {
        BLOCK 1
        %1 = 1
        CALL 0(%1)
        %2 = %0
        %-2 = %2
        RETURN ;
        };
    "
    );
}

#[test]
fn test_global_writes() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_globals_writes",
        req,
        "
        define test (result 0) (define %-1 %0 %2) {
            %0 = 1
            %-1 = %0 
            %2 <- CALL mytest()
        };
        define mytest (param) (result 1) (define %-1 %0 %1)  {
            %0 = 2   
            %1 = 3
            RETURN %-1;
        };
    "
    );
}

#[test]
fn test_memory_store() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_memory_store",
        req,
        "
        define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7 %8 %9) {
        BLOCK 0
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %2 = 8
        %3 = -12345
        STORE FROM %3 OFFSET 0 + %2 ALIGN 3 64
        %5 = 8
        %6 = -12345
        STORE FROM %6 OFFSET 0 + %5 ALIGN 2 32
        %7 = 8
        %8 = -12345
        STORE FROM %8 OFFSET 0 + %7 ALIGN 3 64
        RETURN ;
        };
    "
    );
}

#[test]
fn test_memory_load() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_memory_load",
        req,
        "
       define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7) {
        BLOCK 0
        %0 = 8
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %4 = 8
        %5 = LOAD OFFSET 0 + %4 ALIGN 0
        %6 = 8
        %7 = LOAD OFFSET 0 + %6 ALIGN 0
        KILL %7
        KILL %6
        RETURN ;
       }; 
    "
    );
}

#[test]
fn test_memory_load_different_functions() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_memory_load_different_functions",
        req,
        "
       define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7) {
        BLOCK 0
        %0 = 8
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %2 <- CALL 1 ()
        RETURN ;
       }; 

       define 1 (result 1) (define %0 %1) {
        %1 = 8
        %0 = LOAD OFFSET 0 + %1 ALIGN 0
        RETURN %0;
       };
    "
    );
}

#[test]
fn test_memory_load_different_functions2() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_memory_load_different_functions2",
        req,
        "
       define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7) {
        BLOCK 0
        %0 = 8
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %2 <- CALL 1 ()
        STORE FROM %1 OFFSET 1 + %0 ALIGN 2 32
        %3 <- CALL 1 ()
        RETURN ;
       }; 

       define 1 (result 1) (define %0 %1) {
        %1 = 8
        %0 = LOAD OFFSET 0 + %1 ALIGN 0
        RETURN %0;
       };
    "
    );
}

#[test]
fn fix_broken_pass_args_rg() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 1,
    };
    ir!(
        "fix_broken_pass_args_rg",
        req,
        "
        define 0 (param %0 %1 %2) (result 1) (define %-3 %-2 %-1 %0 %1 %2 %3 %4 %5 %6 %7 %8 %9 %10 %11 %12 %13 %14 %15 %16 %17 %18 %19 %20 %21 %22 %23 %24 %25 %26 %27 %28 %29 %30 %31 %32 %33 %34 %35 %36 %37 %38 %39 %40 %41 %42 %43 %44 %45 %46 %47 %48 %49 %50 %51 %52 %53 %54 %55 %56 %57 %58 %59 %60 %61 %62 %63 %64 %65 %66 %67 %68 %69 %70 %71 %72 %73 %74 %75 %76 %77 %78 %79 %80 %81 %82 %83 %84 %85 %86 %87 %88 %89 %90 %91 %92 %93 %94 %95 %96 %97 %98 %99 %100 %101 %102 %103 %104 %105 %106 %107 %108 %109 %110 %111 %112 %113 %114 %115 %116 %117 %118 %119 %120 %121 %122 %123 %124 %125 %126 %127 %128 %129 %130 %131 %132 %133 %134 %135 %136 %137 %138 %139 %140 %141 %142 %143 %144 %145 %146 %147 %148 %149 %150 %151 %152 %153 %154 %155 %156 %157 %158 %159 %160 %161 %162 %163 %164 %165 %166 %167 %168 %169 %170 %171 %172 %173 %174 %175 %176 %177 %178 %179 %180 %181 %182 %183 %184 %185 %186 %187 %188 %189 %190 %191 %192 %193 %194 %195 %196 %197 %198 %199 %200 %201 %202 %203 %204 %205 %206 %207 %208 %209 %210 %211 %212 %213 %214 %215 %216 %217 %218 %219 %220 %221 %222 %223 %224 %225 %226 %227 %228 %229 %230 %231 %232 %233 %234 %235 %236 %237 %238 %239 %240 %241 %242 %243 %244 %245 %246 %247 %248 %249 %250 %251 %252 %253 %254 %255 %256 %257 %258 %259 %260 %261 %262 %263 %264 %265 %266 %267 %268 %269 %270 %271 %272 %273 %274 %275 %276 %277 %278 %279 %280 %281 %282 %283 %284 %285 %286 %287 %288 %289 %290 %291 %292 %293 %294 %295 %296 %297 %298 %299 %300 %301 %302 %303 %304 %305 %306 %307 %308 %309 %310 %311 %312 %313 %314 %315 %316 %317 %318 %319 %320 %321 %322 %323 %324 %325 %326 %327 %328 %329 %330 %331 %332 %333 %334 %335 %336 %337 %338 %339 %340 %341 %342 %343 %344 %345 %346 %347 %348 %349 %350 %351 %352 %353 %354 %355 %356 %357 %358 %359 %360 %361 %362 %363 %364 %365 %366 %367 %368 %369 %370 %371 %372 %373 %374 %375 %376 %377 %378 %379 %380 %381 %382 %383 %384 %385 %386 %387 %388 %389 %390 %391 %392 %393 %394 %395 %396 %397 %398 %399 %400 %401 %402 %403 %404 %405 %406 %407 %408 %409 %410 %411 %412 %413 %414 %415 %416 %417 %418 %419 %420 %421 %422 %423 %424 %425 %426 %427 %428 %429 %430 %431 %432 %433 %434 %435 %436 %437 %438 %439 %440 %441 %442 %443 %444 %445 %446 %447 %448 %449 %450 %451 %452 %453 %454 %455 %456 %457 %458 %459 %460 %461 %462 %463 %464 %465 %466 %467 %468 %469 %470 %471 %472 %473 %474 %475 %476 %477 %478 %479 %480 %481 %482 %483 %484 %485 %486 %487 %488 %489 %490 %491 %492 %493 %494 %495 %496 %497 %498 %499 %500 %501 %502 %503 %504 %505 %506 %507 %508 %509 %510 %511 %512 %513 %514 %515 %516 %517 %518 %519 %520 %521 %522 %523 %524 %525 %526 %527 %528 %529 %530 %531 %532 %533 %534 %535 %536 %537 %538 %539 %540 %541 %542 %543 %544 %545 %546 %547 %548 %549 %550 %551 %552 %553 %554 %555 %556 %557 %558 %559 %560 %561 %562 %563 %564 %565 %566 %567 %568 %569 %570 %571 %572 %573 %574 %575 %576 %577 %578 %579 %580 %581 %582 %583 %584 %585 %586 %587 %588 %589 %590 %591 %592 %593 %594 %595 %596 %597 %598 %599 %600 %601 %602 %603 %604 %605 %606 %607 %608 %609 %610 %611 %612 %613 %614 %615 %616 %617 %618 %619 %620 %621 %622 %623 %624 %625 %626 %627 %628 %629 %630 %631 %632 %633 %634 %635 %636 %637 %638 %639 %640 %641 %642 %643 %644 %645 %646 %647 %648 %649 %650 %651 %652 %653 %654 %655 %656 %657 %658 %659 %660 %661 %662 %663 %664 %665 %666 %667 %668 %669) {
            BLOCK 0
            %1 = 1
            STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
            %60 = %59 op %5
            %61 <- CALL 959(%60)
            RETURN ;
       }; 
       
       define 959 (param %0) (result 1) (define %-3 %-2 %-1 %0 %1) {
            BLOCK 29121
            %1 = %0
            RETURN %1;
       };
    "
    );
}
