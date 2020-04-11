(module
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (type (;2;) (func (param i32 i32)))
  (type (;3;) (func (param i32)))
  (type (;4;) (func (param i32 i32 i32)))
  (type (;5;) (func (param i32) (result i32)))
  (type (;6;) (func (param i32) (result i64)))
  (type (;7;) (func))
  (type (;8;) (func (param i32 i32 i32 i32)))
  (type (;9;) (func (param i32 i32 i32 i32) (result i32)))
  (type (;10;) (func (param i64 i32) (result i32)))
  (func (;0;) (type 5) (param i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i64)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              i32.const 245
              i32.ge_u
              if  ;; label = @6
                local.get 0
                i32.const -65587
                i32.ge_u
                br_if 4 (;@2;)
                local.get 0
                i32.const 11
                i32.add
                local.tee 0
                i32.const -8
                i32.and
                local.set 5
                i32.const 1049388
                i32.load
                local.tee 8
                i32.eqz
                br_if 1 (;@5;)
                i32.const 0
                local.get 5
                i32.sub
                local.set 6
                block  ;; label = @7
                  block  ;; label = @8
                    block (result i32)  ;; label = @9
                      i32.const 0
                      local.get 0
                      i32.const 8
                      i32.shr_u
                      local.tee 0
                      i32.eqz
                      br_if 0 (;@9;)
                      drop
                      i32.const 31
                      local.get 5
                      i32.const 16777215
                      i32.gt_u
                      br_if 0 (;@9;)
                      drop
                      local.get 5
                      i32.const 6
                      local.get 0
                      i32.clz
                      local.tee 0
                      i32.sub
                      i32.const 31
                      i32.and
                      i32.shr_u
                      i32.const 1
                      i32.and
                      local.get 0
                      i32.const 1
                      i32.shl
                      i32.sub
                      i32.const 62
                      i32.add
                    end
                    local.tee 7
                    i32.const 2
                    i32.shl
                    i32.const 1049656
                    i32.add
                    i32.load
                    local.tee 0
                    if  ;; label = @9
                      local.get 5
                      i32.const 0
                      i32.const 25
                      local.get 7
                      i32.const 1
                      i32.shr_u
                      i32.sub
                      i32.const 31
                      i32.and
                      local.get 7
                      i32.const 31
                      i32.eq
                      select
                      i32.shl
                      local.set 2
                      loop  ;; label = @10
                        block  ;; label = @11
                          local.get 0
                          i32.const 4
                          i32.add
                          i32.load
                          i32.const -8
                          i32.and
                          local.tee 4
                          local.get 5
                          i32.lt_u
                          br_if 0 (;@11;)
                          local.get 4
                          local.get 5
                          i32.sub
                          local.tee 4
                          local.get 6
                          i32.ge_u
                          br_if 0 (;@11;)
                          local.get 0
                          local.set 3
                          local.get 4
                          local.tee 6
                          br_if 0 (;@11;)
                          i32.const 0
                          local.set 6
                          br 3 (;@8;)
                        end
                        local.get 0
                        i32.const 20
                        i32.add
                        i32.load
                        local.tee 4
                        local.get 1
                        local.get 4
                        local.get 0
                        local.get 2
                        i32.const 29
                        i32.shr_u
                        i32.const 4
                        i32.and
                        i32.add
                        i32.const 16
                        i32.add
                        i32.load
                        local.tee 0
                        i32.ne
                        select
                        local.get 1
                        local.get 4
                        select
                        local.set 1
                        local.get 2
                        i32.const 1
                        i32.shl
                        local.set 2
                        local.get 0
                        br_if 0 (;@10;)
                      end
                      local.get 1
                      if  ;; label = @10
                        local.get 1
                        local.set 0
                        br 2 (;@8;)
                      end
                      local.get 3
                      br_if 2 (;@7;)
                    end
                    i32.const 0
                    local.set 3
                    i32.const 2
                    local.get 7
                    i32.const 31
                    i32.and
                    i32.shl
                    local.tee 0
                    i32.const 0
                    local.get 0
                    i32.sub
                    i32.or
                    local.get 8
                    i32.and
                    local.tee 0
                    i32.eqz
                    br_if 3 (;@5;)
                    local.get 0
                    i32.const 0
                    local.get 0
                    i32.sub
                    i32.and
                    i32.ctz
                    i32.const 2
                    i32.shl
                    i32.const 1049656
                    i32.add
                    i32.load
                    local.tee 0
                    i32.eqz
                    br_if 3 (;@5;)
                  end
                  loop  ;; label = @8
                    local.get 0
                    local.get 3
                    local.get 0
                    i32.const 4
                    i32.add
                    i32.load
                    i32.const -8
                    i32.and
                    local.tee 1
                    local.get 5
                    i32.ge_u
                    local.get 1
                    local.get 5
                    i32.sub
                    local.tee 1
                    local.get 6
                    i32.lt_u
                    i32.and
                    local.tee 2
                    select
                    local.set 3
                    local.get 1
                    local.get 6
                    local.get 2
                    select
                    local.set 6
                    local.get 0
                    i32.load offset=16
                    local.tee 1
                    if (result i32)  ;; label = @9
                      local.get 1
                    else
                      local.get 0
                      i32.const 20
                      i32.add
                      i32.load
                    end
                    local.tee 0
                    br_if 0 (;@8;)
                  end
                  local.get 3
                  i32.eqz
                  br_if 2 (;@5;)
                end
                i32.const 1049784
                i32.load
                local.tee 0
                local.get 5
                i32.ge_u
                i32.const 0
                local.get 6
                local.get 0
                local.get 5
                i32.sub
                i32.ge_u
                select
                br_if 1 (;@5;)
                local.get 3
                i32.load offset=24
                local.set 7
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 3
                    local.get 3
                    i32.load offset=12
                    local.tee 1
                    i32.eq
                    if  ;; label = @9
                      local.get 3
                      i32.const 20
                      i32.const 16
                      local.get 3
                      i32.const 20
                      i32.add
                      local.tee 1
                      i32.load
                      local.tee 2
                      select
                      i32.add
                      i32.load
                      local.tee 0
                      br_if 1 (;@8;)
                      i32.const 0
                      local.set 1
                      br 2 (;@7;)
                    end
                    local.get 3
                    i32.load offset=8
                    local.tee 0
                    local.get 1
                    i32.store offset=12
                    local.get 1
                    local.get 0
                    i32.store offset=8
                    br 1 (;@7;)
                  end
                  local.get 1
                  local.get 3
                  i32.const 16
                  i32.add
                  local.get 2
                  select
                  local.set 2
                  loop  ;; label = @8
                    local.get 2
                    local.set 4
                    local.get 0
                    local.tee 1
                    i32.const 20
                    i32.add
                    local.tee 2
                    i32.load
                    local.tee 0
                    i32.eqz
                    if  ;; label = @9
                      local.get 1
                      i32.const 16
                      i32.add
                      local.set 2
                      local.get 1
                      i32.load offset=16
                      local.set 0
                    end
                    local.get 0
                    br_if 0 (;@8;)
                  end
                  local.get 4
                  i32.const 0
                  i32.store
                end
                block  ;; label = @7
                  local.get 7
                  i32.eqz
                  br_if 0 (;@7;)
                  block  ;; label = @8
                    local.get 3
                    local.get 3
                    i32.load offset=28
                    i32.const 2
                    i32.shl
                    i32.const 1049656
                    i32.add
                    local.tee 0
                    i32.load
                    i32.ne
                    if  ;; label = @9
                      local.get 7
                      i32.const 16
                      i32.const 20
                      local.get 7
                      i32.load offset=16
                      local.get 3
                      i32.eq
                      select
                      i32.add
                      local.get 1
                      i32.store
                      local.get 1
                      i32.eqz
                      br_if 2 (;@7;)
                      br 1 (;@8;)
                    end
                    local.get 0
                    local.get 1
                    i32.store
                    local.get 1
                    br_if 0 (;@8;)
                    i32.const 1049388
                    i32.const 1049388
                    i32.load
                    i32.const -2
                    local.get 3
                    i32.load offset=28
                    i32.rotl
                    i32.and
                    i32.store
                    br 1 (;@7;)
                  end
                  local.get 1
                  local.get 7
                  i32.store offset=24
                  local.get 3
                  i32.load offset=16
                  local.tee 0
                  if  ;; label = @8
                    local.get 1
                    local.get 0
                    i32.store offset=16
                    local.get 0
                    local.get 1
                    i32.store offset=24
                  end
                  local.get 3
                  i32.const 20
                  i32.add
                  i32.load
                  local.tee 0
                  i32.eqz
                  br_if 0 (;@7;)
                  local.get 1
                  i32.const 20
                  i32.add
                  local.get 0
                  i32.store
                  local.get 0
                  local.get 1
                  i32.store offset=24
                end
                block  ;; label = @7
                  local.get 6
                  i32.const 16
                  i32.ge_u
                  if  ;; label = @8
                    local.get 3
                    local.get 5
                    i32.const 3
                    i32.or
                    i32.store offset=4
                    local.get 3
                    local.get 5
                    i32.add
                    local.tee 4
                    local.get 6
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    local.get 4
                    local.get 6
                    i32.add
                    local.get 6
                    i32.store
                    local.get 6
                    i32.const 256
                    i32.ge_u
                    if  ;; label = @9
                      local.get 4
                      i64.const 0
                      i64.store offset=16 align=4
                      local.get 4
                      block (result i32)  ;; label = @10
                        i32.const 0
                        local.get 6
                        i32.const 8
                        i32.shr_u
                        local.tee 0
                        i32.eqz
                        br_if 0 (;@10;)
                        drop
                        i32.const 31
                        local.get 6
                        i32.const 16777215
                        i32.gt_u
                        br_if 0 (;@10;)
                        drop
                        local.get 6
                        i32.const 6
                        local.get 0
                        i32.clz
                        local.tee 0
                        i32.sub
                        i32.const 31
                        i32.and
                        i32.shr_u
                        i32.const 1
                        i32.and
                        local.get 0
                        i32.const 1
                        i32.shl
                        i32.sub
                        i32.const 62
                        i32.add
                      end
                      local.tee 0
                      i32.store offset=28
                      local.get 0
                      i32.const 2
                      i32.shl
                      i32.const 1049656
                      i32.add
                      local.set 1
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              i32.const 1049388
                              i32.load
                              local.tee 2
                              i32.const 1
                              local.get 0
                              i32.const 31
                              i32.and
                              i32.shl
                              local.tee 5
                              i32.and
                              if  ;; label = @14
                                local.get 1
                                i32.load
                                local.tee 2
                                i32.const 4
                                i32.add
                                i32.load
                                i32.const -8
                                i32.and
                                local.get 6
                                i32.ne
                                br_if 1 (;@13;)
                                local.get 2
                                local.set 0
                                br 2 (;@12;)
                              end
                              i32.const 1049388
                              local.get 2
                              local.get 5
                              i32.or
                              i32.store
                              local.get 1
                              local.get 4
                              i32.store
                              local.get 4
                              local.get 1
                              i32.store offset=24
                              br 3 (;@10;)
                            end
                            local.get 6
                            i32.const 0
                            i32.const 25
                            local.get 0
                            i32.const 1
                            i32.shr_u
                            i32.sub
                            i32.const 31
                            i32.and
                            local.get 0
                            i32.const 31
                            i32.eq
                            select
                            i32.shl
                            local.set 1
                            loop  ;; label = @13
                              local.get 2
                              local.get 1
                              i32.const 29
                              i32.shr_u
                              i32.const 4
                              i32.and
                              i32.add
                              i32.const 16
                              i32.add
                              local.tee 5
                              i32.load
                              local.tee 0
                              i32.eqz
                              br_if 2 (;@11;)
                              local.get 1
                              i32.const 1
                              i32.shl
                              local.set 1
                              local.get 0
                              local.set 2
                              local.get 0
                              i32.const 4
                              i32.add
                              i32.load
                              i32.const -8
                              i32.and
                              local.get 6
                              i32.ne
                              br_if 0 (;@13;)
                            end
                          end
                          local.get 0
                          i32.load offset=8
                          local.tee 1
                          local.get 4
                          i32.store offset=12
                          local.get 0
                          local.get 4
                          i32.store offset=8
                          local.get 4
                          i32.const 0
                          i32.store offset=24
                          local.get 4
                          local.get 0
                          i32.store offset=12
                          local.get 4
                          local.get 1
                          i32.store offset=8
                          br 4 (;@7;)
                        end
                        local.get 5
                        local.get 4
                        i32.store
                        local.get 4
                        local.get 2
                        i32.store offset=24
                      end
                      local.get 4
                      local.get 4
                      i32.store offset=12
                      local.get 4
                      local.get 4
                      i32.store offset=8
                      br 2 (;@7;)
                    end
                    local.get 6
                    i32.const 3
                    i32.shr_u
                    local.tee 1
                    i32.const 3
                    i32.shl
                    i32.const 1049392
                    i32.add
                    local.set 0
                    block (result i32)  ;; label = @9
                      i32.const 1049384
                      i32.load
                      local.tee 2
                      i32.const 1
                      local.get 1
                      i32.const 31
                      i32.and
                      i32.shl
                      local.tee 1
                      i32.and
                      if  ;; label = @10
                        local.get 0
                        i32.load offset=8
                        br 1 (;@9;)
                      end
                      i32.const 1049384
                      local.get 1
                      local.get 2
                      i32.or
                      i32.store
                      local.get 0
                    end
                    local.set 1
                    local.get 0
                    local.get 4
                    i32.store offset=8
                    local.get 1
                    local.get 4
                    i32.store offset=12
                    local.get 4
                    local.get 0
                    i32.store offset=12
                    local.get 4
                    local.get 1
                    i32.store offset=8
                    br 1 (;@7;)
                  end
                  local.get 3
                  local.get 5
                  local.get 6
                  i32.add
                  local.tee 0
                  i32.const 3
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 3
                  i32.add
                  local.tee 0
                  local.get 0
                  i32.load offset=4
                  i32.const 1
                  i32.or
                  i32.store offset=4
                end
                local.get 3
                i32.const 8
                i32.add
                return
              end
              block  ;; label = @6
                block  ;; label = @7
                  i32.const 1049384
                  i32.load
                  local.tee 1
                  i32.const 16
                  local.get 0
                  i32.const 11
                  i32.add
                  i32.const -8
                  i32.and
                  local.get 0
                  i32.const 11
                  i32.lt_u
                  select
                  local.tee 5
                  i32.const 3
                  i32.shr_u
                  local.tee 6
                  i32.const 31
                  i32.and
                  local.tee 2
                  i32.shr_u
                  local.tee 0
                  i32.const 3
                  i32.and
                  i32.eqz
                  if  ;; label = @8
                    local.get 5
                    i32.const 1049784
                    i32.load
                    i32.le_u
                    br_if 3 (;@5;)
                    local.get 0
                    br_if 1 (;@7;)
                    i32.const 1049388
                    i32.load
                    local.tee 0
                    i32.eqz
                    br_if 3 (;@5;)
                    local.get 0
                    i32.const 0
                    local.get 0
                    i32.sub
                    i32.and
                    i32.ctz
                    i32.const 2
                    i32.shl
                    i32.const 1049656
                    i32.add
                    i32.load
                    local.tee 1
                    i32.const 4
                    i32.add
                    i32.load
                    i32.const -8
                    i32.and
                    local.get 5
                    i32.sub
                    local.set 6
                    local.get 1
                    local.set 2
                    loop  ;; label = @9
                      local.get 1
                      i32.load offset=16
                      local.tee 0
                      i32.eqz
                      if  ;; label = @10
                        local.get 1
                        i32.const 20
                        i32.add
                        i32.load
                        local.tee 0
                        i32.eqz
                        br_if 4 (;@6;)
                      end
                      local.get 0
                      i32.const 4
                      i32.add
                      i32.load
                      i32.const -8
                      i32.and
                      local.get 5
                      i32.sub
                      local.tee 1
                      local.get 6
                      local.get 1
                      local.get 6
                      i32.lt_u
                      local.tee 1
                      select
                      local.set 6
                      local.get 0
                      local.get 2
                      local.get 1
                      select
                      local.set 2
                      local.get 0
                      local.set 1
                      br 0 (;@9;)
                    end
                    unreachable
                  end
                  block  ;; label = @8
                    local.get 0
                    i32.const -1
                    i32.xor
                    i32.const 1
                    i32.and
                    local.get 6
                    i32.add
                    local.tee 0
                    i32.const 3
                    i32.shl
                    i32.const 1049384
                    i32.add
                    local.tee 4
                    i32.const 16
                    i32.add
                    i32.load
                    local.tee 2
                    i32.const 8
                    i32.add
                    local.tee 6
                    i32.load
                    local.tee 3
                    local.get 4
                    i32.const 8
                    i32.add
                    local.tee 4
                    i32.ne
                    if  ;; label = @9
                      local.get 3
                      local.get 4
                      i32.store offset=12
                      local.get 4
                      local.get 3
                      i32.store offset=8
                      br 1 (;@8;)
                    end
                    i32.const 1049384
                    local.get 1
                    i32.const -2
                    local.get 0
                    i32.rotl
                    i32.and
                    i32.store
                  end
                  local.get 2
                  local.get 0
                  i32.const 3
                  i32.shl
                  local.tee 0
                  i32.const 3
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 2
                  i32.add
                  local.tee 0
                  local.get 0
                  i32.load offset=4
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  br 5 (;@2;)
                end
                block  ;; label = @7
                  i32.const 2
                  local.get 2
                  i32.shl
                  local.tee 6
                  i32.const 0
                  local.get 6
                  i32.sub
                  i32.or
                  local.get 0
                  local.get 2
                  i32.shl
                  i32.and
                  local.tee 0
                  i32.const 0
                  local.get 0
                  i32.sub
                  i32.and
                  i32.ctz
                  local.tee 2
                  i32.const 3
                  i32.shl
                  i32.const 1049384
                  i32.add
                  local.tee 3
                  i32.const 16
                  i32.add
                  i32.load
                  local.tee 0
                  i32.const 8
                  i32.add
                  local.tee 4
                  i32.load
                  local.tee 6
                  local.get 3
                  i32.const 8
                  i32.add
                  local.tee 3
                  i32.ne
                  if  ;; label = @8
                    local.get 6
                    local.get 3
                    i32.store offset=12
                    local.get 3
                    local.get 6
                    i32.store offset=8
                    br 1 (;@7;)
                  end
                  i32.const 1049384
                  local.get 1
                  i32.const -2
                  local.get 2
                  i32.rotl
                  i32.and
                  i32.store
                end
                local.get 0
                local.get 5
                i32.const 3
                i32.or
                i32.store offset=4
                local.get 0
                local.get 5
                i32.add
                local.tee 3
                local.get 2
                i32.const 3
                i32.shl
                local.tee 1
                local.get 5
                i32.sub
                local.tee 2
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 0
                local.get 1
                i32.add
                local.get 2
                i32.store
                i32.const 1049784
                i32.load
                local.tee 0
                if  ;; label = @7
                  local.get 0
                  i32.const 3
                  i32.shr_u
                  local.tee 6
                  i32.const 3
                  i32.shl
                  i32.const 1049392
                  i32.add
                  local.set 0
                  i32.const 1049792
                  i32.load
                  local.set 1
                  block (result i32)  ;; label = @8
                    i32.const 1049384
                    i32.load
                    local.tee 5
                    i32.const 1
                    local.get 6
                    i32.const 31
                    i32.and
                    i32.shl
                    local.tee 6
                    i32.and
                    if  ;; label = @9
                      local.get 0
                      i32.load offset=8
                      br 1 (;@8;)
                    end
                    i32.const 1049384
                    local.get 5
                    local.get 6
                    i32.or
                    i32.store
                    local.get 0
                  end
                  local.set 6
                  local.get 0
                  local.get 1
                  i32.store offset=8
                  local.get 6
                  local.get 1
                  i32.store offset=12
                  local.get 1
                  local.get 0
                  i32.store offset=12
                  local.get 1
                  local.get 6
                  i32.store offset=8
                end
                i32.const 1049792
                local.get 3
                i32.store
                i32.const 1049784
                local.get 2
                i32.store
                local.get 4
                return
              end
              local.get 2
              i32.load offset=24
              local.set 7
              block  ;; label = @6
                block  ;; label = @7
                  local.get 2
                  local.get 2
                  i32.load offset=12
                  local.tee 1
                  i32.eq
                  if  ;; label = @8
                    local.get 2
                    i32.const 20
                    i32.const 16
                    local.get 2
                    i32.const 20
                    i32.add
                    local.tee 1
                    i32.load
                    local.tee 3
                    select
                    i32.add
                    i32.load
                    local.tee 0
                    br_if 1 (;@7;)
                    i32.const 0
                    local.set 1
                    br 2 (;@6;)
                  end
                  local.get 2
                  i32.load offset=8
                  local.tee 0
                  local.get 1
                  i32.store offset=12
                  local.get 1
                  local.get 0
                  i32.store offset=8
                  br 1 (;@6;)
                end
                local.get 1
                local.get 2
                i32.const 16
                i32.add
                local.get 3
                select
                local.set 3
                loop  ;; label = @7
                  local.get 3
                  local.set 4
                  local.get 0
                  local.tee 1
                  i32.const 20
                  i32.add
                  local.tee 3
                  i32.load
                  local.tee 0
                  i32.eqz
                  if  ;; label = @8
                    local.get 1
                    i32.const 16
                    i32.add
                    local.set 3
                    local.get 1
                    i32.load offset=16
                    local.set 0
                  end
                  local.get 0
                  br_if 0 (;@7;)
                end
                local.get 4
                i32.const 0
                i32.store
              end
              local.get 7
              i32.eqz
              br_if 2 (;@3;)
              local.get 2
              local.get 2
              i32.load offset=28
              i32.const 2
              i32.shl
              i32.const 1049656
              i32.add
              local.tee 0
              i32.load
              i32.ne
              if  ;; label = @6
                local.get 7
                i32.const 16
                i32.const 20
                local.get 7
                i32.load offset=16
                local.get 2
                i32.eq
                select
                i32.add
                local.get 1
                i32.store
                local.get 1
                i32.eqz
                br_if 3 (;@3;)
                br 2 (;@4;)
              end
              local.get 0
              local.get 1
              i32.store
              local.get 1
              br_if 1 (;@4;)
              i32.const 1049388
              i32.const 1049388
              i32.load
              i32.const -2
              local.get 2
              i32.load offset=28
              i32.rotl
              i32.and
              i32.store
              br 2 (;@3;)
            end
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    i32.const 1049784
                    i32.load
                    local.tee 0
                    local.get 5
                    i32.lt_u
                    if  ;; label = @9
                      i32.const 1049788
                      i32.load
                      local.tee 0
                      local.get 5
                      i32.gt_u
                      br_if 8 (;@1;)
                      i32.const 0
                      local.set 6
                      local.get 5
                      i32.const 65583
                      i32.add
                      local.tee 0
                      i32.const 16
                      i32.shr_u
                      memory.grow
                      local.tee 1
                      i32.const -1
                      i32.eq
                      br_if 7 (;@2;)
                      local.get 1
                      i32.const 16
                      i32.shl
                      local.tee 4
                      i32.eqz
                      br_if 7 (;@2;)
                      i32.const 1049800
                      local.get 0
                      i32.const -65536
                      i32.and
                      local.tee 7
                      i32.const 1049800
                      i32.load
                      i32.add
                      local.tee 0
                      i32.store
                      i32.const 1049804
                      i32.const 1049804
                      i32.load
                      local.tee 1
                      local.get 0
                      local.get 1
                      local.get 0
                      i32.gt_u
                      select
                      i32.store
                      i32.const 1049796
                      i32.load
                      local.tee 3
                      i32.eqz
                      br_if 1 (;@8;)
                      i32.const 1049808
                      local.set 0
                      loop  ;; label = @10
                        local.get 0
                        i32.load
                        local.tee 1
                        local.get 0
                        i32.load offset=4
                        local.tee 2
                        i32.add
                        local.get 4
                        i32.eq
                        br_if 3 (;@7;)
                        local.get 0
                        i32.load offset=8
                        local.tee 0
                        br_if 0 (;@10;)
                      end
                      br 3 (;@6;)
                    end
                    i32.const 1049792
                    i32.load
                    local.set 1
                    block (result i32)  ;; label = @9
                      local.get 0
                      local.get 5
                      i32.sub
                      local.tee 2
                      i32.const 15
                      i32.le_u
                      if  ;; label = @10
                        i32.const 1049792
                        i32.const 0
                        i32.store
                        i32.const 1049784
                        i32.const 0
                        i32.store
                        local.get 1
                        local.get 0
                        i32.const 3
                        i32.or
                        i32.store offset=4
                        local.get 0
                        local.get 1
                        i32.add
                        local.tee 2
                        i32.const 4
                        i32.add
                        local.set 0
                        local.get 2
                        i32.load offset=4
                        i32.const 1
                        i32.or
                        br 1 (;@9;)
                      end
                      i32.const 1049784
                      local.get 2
                      i32.store
                      i32.const 1049792
                      local.get 1
                      local.get 5
                      i32.add
                      local.tee 6
                      i32.store
                      local.get 6
                      local.get 2
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      local.get 0
                      local.get 1
                      i32.add
                      local.get 2
                      i32.store
                      local.get 1
                      i32.const 4
                      i32.add
                      local.set 0
                      local.get 5
                      i32.const 3
                      i32.or
                    end
                    local.set 2
                    local.get 0
                    local.get 2
                    i32.store
                    local.get 1
                    i32.const 8
                    i32.add
                    return
                  end
                  i32.const 1049828
                  i32.load
                  local.tee 0
                  i32.const 0
                  local.get 0
                  local.get 4
                  i32.le_u
                  select
                  i32.eqz
                  if  ;; label = @8
                    i32.const 1049828
                    local.get 4
                    i32.store
                  end
                  i32.const 1049832
                  i32.const 4095
                  i32.store
                  i32.const 1049808
                  local.get 4
                  i32.store
                  i32.const 1049820
                  i32.const 0
                  i32.store
                  i32.const 1049812
                  local.get 7
                  i32.store
                  i32.const 1049404
                  i32.const 1049392
                  i32.store
                  i32.const 1049412
                  i32.const 1049400
                  i32.store
                  i32.const 1049400
                  i32.const 1049392
                  i32.store
                  i32.const 1049420
                  i32.const 1049408
                  i32.store
                  i32.const 1049408
                  i32.const 1049400
                  i32.store
                  i32.const 1049428
                  i32.const 1049416
                  i32.store
                  i32.const 1049416
                  i32.const 1049408
                  i32.store
                  i32.const 1049436
                  i32.const 1049424
                  i32.store
                  i32.const 1049424
                  i32.const 1049416
                  i32.store
                  i32.const 1049444
                  i32.const 1049432
                  i32.store
                  i32.const 1049432
                  i32.const 1049424
                  i32.store
                  i32.const 1049452
                  i32.const 1049440
                  i32.store
                  i32.const 1049440
                  i32.const 1049432
                  i32.store
                  i32.const 1049460
                  i32.const 1049448
                  i32.store
                  i32.const 1049448
                  i32.const 1049440
                  i32.store
                  i32.const 1049468
                  i32.const 1049456
                  i32.store
                  i32.const 1049456
                  i32.const 1049448
                  i32.store
                  i32.const 1049464
                  i32.const 1049456
                  i32.store
                  i32.const 1049476
                  i32.const 1049464
                  i32.store
                  i32.const 1049472
                  i32.const 1049464
                  i32.store
                  i32.const 1049484
                  i32.const 1049472
                  i32.store
                  i32.const 1049480
                  i32.const 1049472
                  i32.store
                  i32.const 1049492
                  i32.const 1049480
                  i32.store
                  i32.const 1049488
                  i32.const 1049480
                  i32.store
                  i32.const 1049500
                  i32.const 1049488
                  i32.store
                  i32.const 1049496
                  i32.const 1049488
                  i32.store
                  i32.const 1049508
                  i32.const 1049496
                  i32.store
                  i32.const 1049504
                  i32.const 1049496
                  i32.store
                  i32.const 1049516
                  i32.const 1049504
                  i32.store
                  i32.const 1049512
                  i32.const 1049504
                  i32.store
                  i32.const 1049524
                  i32.const 1049512
                  i32.store
                  i32.const 1049520
                  i32.const 1049512
                  i32.store
                  i32.const 1049532
                  i32.const 1049520
                  i32.store
                  i32.const 1049540
                  i32.const 1049528
                  i32.store
                  i32.const 1049528
                  i32.const 1049520
                  i32.store
                  i32.const 1049548
                  i32.const 1049536
                  i32.store
                  i32.const 1049536
                  i32.const 1049528
                  i32.store
                  i32.const 1049556
                  i32.const 1049544
                  i32.store
                  i32.const 1049544
                  i32.const 1049536
                  i32.store
                  i32.const 1049564
                  i32.const 1049552
                  i32.store
                  i32.const 1049552
                  i32.const 1049544
                  i32.store
                  i32.const 1049572
                  i32.const 1049560
                  i32.store
                  i32.const 1049560
                  i32.const 1049552
                  i32.store
                  i32.const 1049580
                  i32.const 1049568
                  i32.store
                  i32.const 1049568
                  i32.const 1049560
                  i32.store
                  i32.const 1049588
                  i32.const 1049576
                  i32.store
                  i32.const 1049576
                  i32.const 1049568
                  i32.store
                  i32.const 1049596
                  i32.const 1049584
                  i32.store
                  i32.const 1049584
                  i32.const 1049576
                  i32.store
                  i32.const 1049604
                  i32.const 1049592
                  i32.store
                  i32.const 1049592
                  i32.const 1049584
                  i32.store
                  i32.const 1049612
                  i32.const 1049600
                  i32.store
                  i32.const 1049600
                  i32.const 1049592
                  i32.store
                  i32.const 1049620
                  i32.const 1049608
                  i32.store
                  i32.const 1049608
                  i32.const 1049600
                  i32.store
                  i32.const 1049628
                  i32.const 1049616
                  i32.store
                  i32.const 1049616
                  i32.const 1049608
                  i32.store
                  i32.const 1049636
                  i32.const 1049624
                  i32.store
                  i32.const 1049624
                  i32.const 1049616
                  i32.store
                  i32.const 1049644
                  i32.const 1049632
                  i32.store
                  i32.const 1049632
                  i32.const 1049624
                  i32.store
                  i32.const 1049652
                  i32.const 1049640
                  i32.store
                  i32.const 1049640
                  i32.const 1049632
                  i32.store
                  i32.const 1049796
                  local.get 4
                  i32.store
                  i32.const 1049648
                  i32.const 1049640
                  i32.store
                  i32.const 1049788
                  local.get 7
                  i32.const -40
                  i32.add
                  local.tee 0
                  i32.store
                  local.get 4
                  local.get 0
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 4
                  i32.add
                  i32.const 40
                  i32.store offset=4
                  i32.const 1049824
                  i32.const 2097152
                  i32.store
                  br 2 (;@5;)
                end
                local.get 0
                i32.const 12
                i32.add
                i32.load
                local.get 4
                local.get 3
                i32.le_u
                i32.or
                local.get 1
                local.get 3
                i32.gt_u
                i32.or
                br_if 0 (;@6;)
                local.get 0
                local.get 2
                local.get 7
                i32.add
                i32.store offset=4
                i32.const 1049796
                i32.const 1049796
                i32.load
                local.tee 0
                i32.const 15
                i32.add
                i32.const -8
                i32.and
                local.tee 1
                i32.const -8
                i32.add
                i32.store
                i32.const 1049788
                i32.const 1049788
                i32.load
                local.get 7
                i32.add
                local.tee 2
                local.get 0
                local.get 1
                i32.sub
                i32.add
                i32.const 8
                i32.add
                local.tee 3
                i32.store
                local.get 1
                i32.const -4
                i32.add
                local.get 3
                i32.const 1
                i32.or
                i32.store
                local.get 0
                local.get 2
                i32.add
                i32.const 40
                i32.store offset=4
                i32.const 1049824
                i32.const 2097152
                i32.store
                br 1 (;@5;)
              end
              i32.const 1049828
              i32.const 1049828
              i32.load
              local.tee 0
              local.get 4
              local.get 0
              local.get 4
              i32.lt_u
              select
              i32.store
              local.get 4
              local.get 7
              i32.add
              local.set 2
              i32.const 1049808
              local.set 0
              block  ;; label = @6
                loop  ;; label = @7
                  local.get 2
                  local.get 0
                  i32.load
                  i32.ne
                  if  ;; label = @8
                    local.get 0
                    i32.load offset=8
                    local.tee 0
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                end
                local.get 0
                i32.const 12
                i32.add
                i32.load
                br_if 0 (;@6;)
                local.get 0
                local.get 4
                i32.store
                local.get 0
                local.get 0
                i32.load offset=4
                local.get 7
                i32.add
                i32.store offset=4
                local.get 4
                local.get 5
                i32.const 3
                i32.or
                i32.store offset=4
                local.get 4
                local.get 5
                i32.add
                local.set 0
                local.get 2
                local.get 4
                i32.sub
                local.get 5
                i32.sub
                local.set 5
                block  ;; label = @7
                  block  ;; label = @8
                    local.get 2
                    i32.const 1049796
                    i32.load
                    i32.ne
                    if  ;; label = @9
                      i32.const 1049792
                      i32.load
                      local.get 2
                      i32.eq
                      br_if 1 (;@8;)
                      local.get 2
                      i32.const 4
                      i32.add
                      i32.load
                      local.tee 1
                      i32.const 3
                      i32.and
                      i32.const 1
                      i32.eq
                      if  ;; label = @10
                        local.get 2
                        local.get 1
                        i32.const -8
                        i32.and
                        local.tee 1
                        call 8
                        local.get 1
                        local.get 5
                        i32.add
                        local.set 5
                        local.get 1
                        local.get 2
                        i32.add
                        local.set 2
                      end
                      local.get 2
                      local.get 2
                      i32.load offset=4
                      i32.const -2
                      i32.and
                      i32.store offset=4
                      local.get 0
                      local.get 5
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      local.get 0
                      local.get 5
                      i32.add
                      local.get 5
                      i32.store
                      local.get 5
                      i32.const 256
                      i32.ge_u
                      if  ;; label = @10
                        local.get 0
                        i64.const 0
                        i64.store offset=16 align=4
                        local.get 0
                        block (result i32)  ;; label = @11
                          i32.const 0
                          local.get 5
                          i32.const 8
                          i32.shr_u
                          local.tee 1
                          i32.eqz
                          br_if 0 (;@11;)
                          drop
                          i32.const 31
                          local.get 5
                          i32.const 16777215
                          i32.gt_u
                          br_if 0 (;@11;)
                          drop
                          local.get 5
                          i32.const 6
                          local.get 1
                          i32.clz
                          local.tee 1
                          i32.sub
                          i32.const 31
                          i32.and
                          i32.shr_u
                          i32.const 1
                          i32.and
                          local.get 1
                          i32.const 1
                          i32.shl
                          i32.sub
                          i32.const 62
                          i32.add
                        end
                        local.tee 1
                        i32.store offset=28
                        local.get 1
                        i32.const 2
                        i32.shl
                        i32.const 1049656
                        i32.add
                        local.set 2
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              block  ;; label = @14
                                i32.const 1049388
                                i32.load
                                local.tee 6
                                i32.const 1
                                local.get 1
                                i32.const 31
                                i32.and
                                i32.shl
                                local.tee 3
                                i32.and
                                if  ;; label = @15
                                  local.get 2
                                  i32.load
                                  local.tee 2
                                  i32.const 4
                                  i32.add
                                  i32.load
                                  i32.const -8
                                  i32.and
                                  local.get 5
                                  i32.ne
                                  br_if 1 (;@14;)
                                  local.get 2
                                  local.set 6
                                  br 2 (;@13;)
                                end
                                i32.const 1049388
                                local.get 3
                                local.get 6
                                i32.or
                                i32.store
                                local.get 2
                                local.get 0
                                i32.store
                                br 3 (;@11;)
                              end
                              local.get 5
                              i32.const 0
                              i32.const 25
                              local.get 1
                              i32.const 1
                              i32.shr_u
                              i32.sub
                              i32.const 31
                              i32.and
                              local.get 1
                              i32.const 31
                              i32.eq
                              select
                              i32.shl
                              local.set 1
                              loop  ;; label = @14
                                local.get 2
                                local.get 1
                                i32.const 29
                                i32.shr_u
                                i32.const 4
                                i32.and
                                i32.add
                                i32.const 16
                                i32.add
                                local.tee 3
                                i32.load
                                local.tee 6
                                i32.eqz
                                br_if 2 (;@12;)
                                local.get 1
                                i32.const 1
                                i32.shl
                                local.set 1
                                local.get 6
                                local.tee 2
                                i32.const 4
                                i32.add
                                i32.load
                                i32.const -8
                                i32.and
                                local.get 5
                                i32.ne
                                br_if 0 (;@14;)
                              end
                            end
                            local.get 6
                            i32.load offset=8
                            local.tee 1
                            local.get 0
                            i32.store offset=12
                            local.get 6
                            local.get 0
                            i32.store offset=8
                            local.get 0
                            i32.const 0
                            i32.store offset=24
                            local.get 0
                            local.get 6
                            i32.store offset=12
                            local.get 0
                            local.get 1
                            i32.store offset=8
                            br 5 (;@7;)
                          end
                          local.get 3
                          local.get 0
                          i32.store
                        end
                        local.get 0
                        local.get 2
                        i32.store offset=24
                        local.get 0
                        local.get 0
                        i32.store offset=12
                        local.get 0
                        local.get 0
                        i32.store offset=8
                        br 3 (;@7;)
                      end
                      local.get 5
                      i32.const 3
                      i32.shr_u
                      local.tee 2
                      i32.const 3
                      i32.shl
                      i32.const 1049392
                      i32.add
                      local.set 1
                      block (result i32)  ;; label = @10
                        i32.const 1049384
                        i32.load
                        local.tee 6
                        i32.const 1
                        local.get 2
                        i32.const 31
                        i32.and
                        i32.shl
                        local.tee 2
                        i32.and
                        if  ;; label = @11
                          local.get 1
                          i32.load offset=8
                          br 1 (;@10;)
                        end
                        i32.const 1049384
                        local.get 2
                        local.get 6
                        i32.or
                        i32.store
                        local.get 1
                      end
                      local.set 2
                      local.get 1
                      local.get 0
                      i32.store offset=8
                      local.get 2
                      local.get 0
                      i32.store offset=12
                      local.get 0
                      local.get 1
                      i32.store offset=12
                      local.get 0
                      local.get 2
                      i32.store offset=8
                      br 2 (;@7;)
                    end
                    i32.const 1049796
                    local.get 0
                    i32.store
                    i32.const 1049788
                    i32.const 1049788
                    i32.load
                    local.get 5
                    i32.add
                    local.tee 1
                    i32.store
                    local.get 0
                    local.get 1
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    br 1 (;@7;)
                  end
                  i32.const 1049792
                  local.get 0
                  i32.store
                  i32.const 1049784
                  i32.const 1049784
                  i32.load
                  local.get 5
                  i32.add
                  local.tee 1
                  i32.store
                  local.get 0
                  local.get 1
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 1
                  i32.add
                  local.get 1
                  i32.store
                end
                local.get 4
                i32.const 8
                i32.add
                return
              end
              i32.const 1049808
              local.set 0
              loop  ;; label = @6
                block  ;; label = @7
                  local.get 0
                  i32.load
                  local.tee 1
                  local.get 3
                  i32.le_u
                  if  ;; label = @8
                    local.get 1
                    local.get 0
                    i32.load offset=4
                    i32.add
                    local.tee 2
                    local.get 3
                    i32.gt_u
                    br_if 1 (;@7;)
                  end
                  local.get 0
                  i32.load offset=8
                  local.set 0
                  br 1 (;@6;)
                end
              end
              i32.const 1049796
              local.get 4
              i32.store
              i32.const 1049788
              local.get 7
              i32.const -40
              i32.add
              local.tee 0
              i32.store
              local.get 4
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 4
              i32.add
              i32.const 40
              i32.store offset=4
              i32.const 1049824
              i32.const 2097152
              i32.store
              local.get 3
              local.get 2
              i32.const -32
              i32.add
              i32.const -8
              i32.and
              i32.const -8
              i32.add
              local.tee 0
              local.get 0
              local.get 3
              i32.const 16
              i32.add
              i32.lt_u
              select
              local.tee 1
              i32.const 27
              i32.store offset=4
              i32.const 1049808
              i64.load align=4
              local.set 9
              local.get 1
              i32.const 16
              i32.add
              i32.const 1049816
              i64.load align=4
              i64.store align=4
              local.get 1
              local.get 9
              i64.store offset=8 align=4
              i32.const 1049820
              i32.const 0
              i32.store
              i32.const 1049812
              local.get 7
              i32.store
              i32.const 1049808
              local.get 4
              i32.store
              i32.const 1049816
              local.get 1
              i32.const 8
              i32.add
              i32.store
              local.get 1
              i32.const 28
              i32.add
              local.set 0
              loop  ;; label = @6
                local.get 0
                i32.const 7
                i32.store
                local.get 2
                local.get 0
                i32.const 4
                i32.add
                local.tee 0
                i32.gt_u
                br_if 0 (;@6;)
              end
              local.get 1
              local.get 3
              i32.eq
              br_if 0 (;@5;)
              local.get 1
              local.get 1
              i32.load offset=4
              i32.const -2
              i32.and
              i32.store offset=4
              local.get 3
              local.get 1
              local.get 3
              i32.sub
              local.tee 4
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 1
              local.get 4
              i32.store
              local.get 4
              i32.const 256
              i32.ge_u
              if  ;; label = @6
                local.get 3
                i64.const 0
                i64.store offset=16 align=4
                local.get 3
                i32.const 28
                i32.add
                block (result i32)  ;; label = @7
                  i32.const 0
                  local.get 4
                  i32.const 8
                  i32.shr_u
                  local.tee 0
                  i32.eqz
                  br_if 0 (;@7;)
                  drop
                  i32.const 31
                  local.get 4
                  i32.const 16777215
                  i32.gt_u
                  br_if 0 (;@7;)
                  drop
                  local.get 4
                  i32.const 6
                  local.get 0
                  i32.clz
                  local.tee 0
                  i32.sub
                  i32.const 31
                  i32.and
                  i32.shr_u
                  i32.const 1
                  i32.and
                  local.get 0
                  i32.const 1
                  i32.shl
                  i32.sub
                  i32.const 62
                  i32.add
                end
                local.tee 0
                i32.store
                local.get 0
                i32.const 2
                i32.shl
                i32.const 1049656
                i32.add
                local.set 1
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        i32.const 1049388
                        i32.load
                        local.tee 2
                        i32.const 1
                        local.get 0
                        i32.const 31
                        i32.and
                        i32.shl
                        local.tee 7
                        i32.and
                        if  ;; label = @11
                          local.get 1
                          i32.load
                          local.tee 1
                          i32.const 4
                          i32.add
                          i32.load
                          i32.const -8
                          i32.and
                          local.get 4
                          i32.ne
                          br_if 1 (;@10;)
                          local.get 1
                          local.set 0
                          br 2 (;@9;)
                        end
                        i32.const 1049388
                        local.get 2
                        local.get 7
                        i32.or
                        i32.store
                        local.get 1
                        local.get 3
                        i32.store
                        br 3 (;@7;)
                      end
                      local.get 4
                      i32.const 0
                      i32.const 25
                      local.get 0
                      i32.const 1
                      i32.shr_u
                      i32.sub
                      i32.const 31
                      i32.and
                      local.get 0
                      i32.const 31
                      i32.eq
                      select
                      i32.shl
                      local.set 2
                      loop  ;; label = @10
                        local.get 1
                        local.get 2
                        i32.const 29
                        i32.shr_u
                        i32.const 4
                        i32.and
                        i32.add
                        i32.const 16
                        i32.add
                        local.tee 7
                        i32.load
                        local.tee 0
                        i32.eqz
                        br_if 2 (;@8;)
                        local.get 2
                        i32.const 1
                        i32.shl
                        local.set 2
                        local.get 0
                        local.set 1
                        local.get 0
                        i32.const 4
                        i32.add
                        i32.load
                        i32.const -8
                        i32.and
                        local.get 4
                        i32.ne
                        br_if 0 (;@10;)
                      end
                    end
                    local.get 0
                    i32.load offset=8
                    local.tee 1
                    local.get 3
                    i32.store offset=12
                    local.get 0
                    local.get 3
                    i32.store offset=8
                    local.get 3
                    i32.const 24
                    i32.add
                    i32.const 0
                    i32.store
                    local.get 3
                    local.get 0
                    i32.store offset=12
                    local.get 3
                    local.get 1
                    i32.store offset=8
                    br 3 (;@5;)
                  end
                  local.get 7
                  local.get 3
                  i32.store
                end
                local.get 3
                i32.const 24
                i32.add
                local.get 1
                i32.store
                local.get 3
                local.get 3
                i32.store offset=12
                local.get 3
                local.get 3
                i32.store offset=8
                br 1 (;@5;)
              end
              local.get 4
              i32.const 3
              i32.shr_u
              local.tee 1
              i32.const 3
              i32.shl
              i32.const 1049392
              i32.add
              local.set 0
              block (result i32)  ;; label = @6
                i32.const 1049384
                i32.load
                local.tee 2
                i32.const 1
                local.get 1
                i32.const 31
                i32.and
                i32.shl
                local.tee 1
                i32.and
                if  ;; label = @7
                  local.get 0
                  i32.load offset=8
                  br 1 (;@6;)
                end
                i32.const 1049384
                local.get 1
                local.get 2
                i32.or
                i32.store
                local.get 0
              end
              local.set 1
              local.get 0
              local.get 3
              i32.store offset=8
              local.get 1
              local.get 3
              i32.store offset=12
              local.get 3
              local.get 0
              i32.store offset=12
              local.get 3
              local.get 1
              i32.store offset=8
            end
            i32.const 1049788
            i32.load
            local.tee 0
            local.get 5
            i32.le_u
            br_if 2 (;@2;)
            br 3 (;@1;)
          end
          local.get 1
          local.get 7
          i32.store offset=24
          local.get 2
          i32.load offset=16
          local.tee 0
          if  ;; label = @4
            local.get 1
            local.get 0
            i32.store offset=16
            local.get 0
            local.get 1
            i32.store offset=24
          end
          local.get 2
          i32.const 20
          i32.add
          i32.load
          local.tee 0
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          i32.const 20
          i32.add
          local.get 0
          i32.store
          local.get 0
          local.get 1
          i32.store offset=24
        end
        block  ;; label = @3
          local.get 6
          i32.const 16
          i32.ge_u
          if  ;; label = @4
            local.get 2
            local.get 5
            i32.const 3
            i32.or
            i32.store offset=4
            local.get 2
            local.get 5
            i32.add
            local.tee 3
            local.get 6
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 3
            local.get 6
            i32.add
            local.get 6
            i32.store
            i32.const 1049784
            i32.load
            local.tee 0
            if  ;; label = @5
              local.get 0
              i32.const 3
              i32.shr_u
              local.tee 4
              i32.const 3
              i32.shl
              i32.const 1049392
              i32.add
              local.set 0
              i32.const 1049792
              i32.load
              local.set 1
              block (result i32)  ;; label = @6
                i32.const 1049384
                i32.load
                local.tee 5
                i32.const 1
                local.get 4
                i32.const 31
                i32.and
                i32.shl
                local.tee 4
                i32.and
                if  ;; label = @7
                  local.get 0
                  i32.load offset=8
                  br 1 (;@6;)
                end
                i32.const 1049384
                local.get 4
                local.get 5
                i32.or
                i32.store
                local.get 0
              end
              local.set 4
              local.get 0
              local.get 1
              i32.store offset=8
              local.get 4
              local.get 1
              i32.store offset=12
              local.get 1
              local.get 0
              i32.store offset=12
              local.get 1
              local.get 4
              i32.store offset=8
            end
            i32.const 1049792
            local.get 3
            i32.store
            i32.const 1049784
            local.get 6
            i32.store
            br 1 (;@3;)
          end
          local.get 2
          local.get 5
          local.get 6
          i32.add
          local.tee 0
          i32.const 3
          i32.or
          i32.store offset=4
          local.get 0
          local.get 2
          i32.add
          local.tee 0
          local.get 0
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
        end
        local.get 2
        i32.const 8
        i32.add
        return
      end
      local.get 6
      return
    end
    i32.const 1049788
    local.get 0
    local.get 5
    i32.sub
    local.tee 1
    i32.store
    i32.const 1049796
    i32.const 1049796
    i32.load
    local.tee 0
    local.get 5
    i32.add
    local.tee 2
    i32.store
    local.get 2
    local.get 1
    i32.const 1
    i32.or
    i32.store offset=4
    local.get 0
    local.get 5
    i32.const 3
    i32.or
    i32.store offset=4
    local.get 0
    i32.const 8
    i32.add)
  (func (;1;) (type 3) (param i32)
    (local i32 i32 i32 i32 i32)
    local.get 0
    i32.const -8
    i32.add
    local.tee 1
    local.get 0
    i32.const -4
    i32.add
    i32.load
    local.tee 3
    i32.const -8
    i32.and
    local.tee 0
    i32.add
    local.set 2
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 3
            i32.const 1
            i32.and
            br_if 0 (;@4;)
            local.get 3
            i32.const 3
            i32.and
            i32.eqz
            br_if 1 (;@3;)
            local.get 1
            i32.load
            local.tee 3
            local.get 0
            i32.add
            local.set 0
            local.get 1
            local.get 3
            i32.sub
            local.tee 1
            i32.const 1049792
            i32.load
            i32.eq
            if  ;; label = @5
              local.get 2
              i32.load offset=4
              i32.const 3
              i32.and
              i32.const 3
              i32.ne
              br_if 1 (;@4;)
              i32.const 1049784
              local.get 0
              i32.store
              local.get 2
              local.get 2
              i32.load offset=4
              i32.const -2
              i32.and
              i32.store offset=4
              local.get 1
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 1
              i32.add
              local.get 0
              i32.store
              return
            end
            local.get 1
            local.get 3
            call 8
          end
          block  ;; label = @4
            local.get 2
            i32.const 4
            i32.add
            local.tee 4
            i32.load
            local.tee 3
            i32.const 2
            i32.and
            if  ;; label = @5
              local.get 4
              local.get 3
              i32.const -2
              i32.and
              i32.store
              local.get 1
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 1
              i32.add
              local.get 0
              i32.store
              br 1 (;@4;)
            end
            block  ;; label = @5
              local.get 2
              i32.const 1049796
              i32.load
              i32.ne
              if  ;; label = @6
                i32.const 1049792
                i32.load
                local.get 2
                i32.eq
                br_if 1 (;@5;)
                local.get 2
                local.get 3
                i32.const -8
                i32.and
                local.tee 2
                call 8
                local.get 1
                local.get 0
                local.get 2
                i32.add
                local.tee 0
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 0
                local.get 1
                i32.add
                local.get 0
                i32.store
                local.get 1
                i32.const 1049792
                i32.load
                i32.ne
                br_if 2 (;@4;)
                i32.const 1049784
                local.get 0
                i32.store
                return
              end
              i32.const 1049796
              local.get 1
              i32.store
              i32.const 1049788
              i32.const 1049788
              i32.load
              local.get 0
              i32.add
              local.tee 0
              i32.store
              local.get 1
              local.get 0
              i32.const 1
              i32.or
              i32.store offset=4
              i32.const 1049792
              i32.load
              local.get 1
              i32.eq
              if  ;; label = @6
                i32.const 1049784
                i32.const 0
                i32.store
                i32.const 1049792
                i32.const 0
                i32.store
              end
              i32.const 1049824
              i32.load
              local.tee 2
              local.get 0
              i32.ge_u
              br_if 2 (;@3;)
              i32.const 1049796
              i32.load
              local.tee 0
              i32.eqz
              br_if 2 (;@3;)
              block  ;; label = @6
                i32.const 1049788
                i32.load
                local.tee 3
                i32.const 41
                i32.lt_u
                br_if 0 (;@6;)
                i32.const 1049808
                local.set 1
                loop  ;; label = @7
                  local.get 1
                  i32.load
                  local.tee 4
                  local.get 0
                  i32.le_u
                  if  ;; label = @8
                    local.get 4
                    local.get 1
                    i32.load offset=4
                    i32.add
                    local.get 0
                    i32.gt_u
                    br_if 2 (;@6;)
                  end
                  local.get 1
                  i32.load offset=8
                  local.tee 1
                  br_if 0 (;@7;)
                end
              end
              i32.const 1049832
              block (result i32)  ;; label = @6
                i32.const 4095
                i32.const 1049816
                i32.load
                local.tee 0
                i32.eqz
                br_if 0 (;@6;)
                drop
                i32.const 0
                local.set 1
                loop  ;; label = @7
                  local.get 1
                  i32.const 1
                  i32.add
                  local.set 1
                  local.get 0
                  i32.load offset=8
                  local.tee 0
                  br_if 0 (;@7;)
                end
                local.get 1
                i32.const 4095
                local.get 1
                i32.const 4095
                i32.gt_u
                select
              end
              i32.store
              local.get 3
              local.get 2
              i32.le_u
              br_if 2 (;@3;)
              i32.const 1049824
              i32.const -1
              i32.store
              return
            end
            i32.const 1049792
            local.get 1
            i32.store
            i32.const 1049784
            i32.const 1049784
            i32.load
            local.get 0
            i32.add
            local.tee 0
            i32.store
            local.get 1
            local.get 0
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 0
            local.get 1
            i32.add
            local.get 0
            i32.store
            return
          end
          local.get 0
          i32.const 256
          i32.lt_u
          br_if 1 (;@2;)
          local.get 1
          i64.const 0
          i64.store offset=16 align=4
          local.get 1
          i32.const 28
          i32.add
          block (result i32)  ;; label = @4
            i32.const 0
            local.get 0
            i32.const 8
            i32.shr_u
            local.tee 2
            i32.eqz
            br_if 0 (;@4;)
            drop
            i32.const 31
            local.get 0
            i32.const 16777215
            i32.gt_u
            br_if 0 (;@4;)
            drop
            local.get 0
            i32.const 6
            local.get 2
            i32.clz
            local.tee 2
            i32.sub
            i32.const 31
            i32.and
            i32.shr_u
            i32.const 1
            i32.and
            local.get 2
            i32.const 1
            i32.shl
            i32.sub
            i32.const 62
            i32.add
          end
          local.tee 3
          i32.store
          local.get 3
          i32.const 2
          i32.shl
          i32.const 1049656
          i32.add
          local.set 2
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    i32.const 1049388
                    i32.load
                    local.tee 4
                    i32.const 1
                    local.get 3
                    i32.const 31
                    i32.and
                    i32.shl
                    local.tee 5
                    i32.and
                    if  ;; label = @9
                      local.get 2
                      i32.load
                      local.tee 2
                      i32.const 4
                      i32.add
                      i32.load
                      i32.const -8
                      i32.and
                      local.get 0
                      i32.ne
                      br_if 1 (;@8;)
                      local.get 2
                      local.set 3
                      br 2 (;@7;)
                    end
                    i32.const 1049388
                    local.get 4
                    local.get 5
                    i32.or
                    i32.store
                    local.get 2
                    local.get 1
                    i32.store
                    br 3 (;@5;)
                  end
                  local.get 0
                  i32.const 0
                  i32.const 25
                  local.get 3
                  i32.const 1
                  i32.shr_u
                  i32.sub
                  i32.const 31
                  i32.and
                  local.get 3
                  i32.const 31
                  i32.eq
                  select
                  i32.shl
                  local.set 4
                  loop  ;; label = @8
                    local.get 2
                    local.get 4
                    i32.const 29
                    i32.shr_u
                    i32.const 4
                    i32.and
                    i32.add
                    i32.const 16
                    i32.add
                    local.tee 5
                    i32.load
                    local.tee 3
                    i32.eqz
                    br_if 2 (;@6;)
                    local.get 4
                    i32.const 1
                    i32.shl
                    local.set 4
                    local.get 3
                    local.tee 2
                    i32.const 4
                    i32.add
                    i32.load
                    i32.const -8
                    i32.and
                    local.get 0
                    i32.ne
                    br_if 0 (;@8;)
                  end
                end
                local.get 3
                i32.load offset=8
                local.tee 0
                local.get 1
                i32.store offset=12
                local.get 3
                local.get 1
                i32.store offset=8
                local.get 1
                i32.const 24
                i32.add
                i32.const 0
                i32.store
                local.get 1
                local.get 3
                i32.store offset=12
                local.get 1
                local.get 0
                i32.store offset=8
                br 2 (;@4;)
              end
              local.get 5
              local.get 1
              i32.store
            end
            local.get 1
            i32.const 24
            i32.add
            local.get 2
            i32.store
            local.get 1
            local.get 1
            i32.store offset=12
            local.get 1
            local.get 1
            i32.store offset=8
          end
          i32.const 1049832
          i32.const 1049832
          i32.load
          i32.const -1
          i32.add
          local.tee 0
          i32.store
          local.get 0
          i32.eqz
          br_if 2 (;@1;)
        end
        return
      end
      local.get 0
      i32.const 3
      i32.shr_u
      local.tee 2
      i32.const 3
      i32.shl
      i32.const 1049392
      i32.add
      local.set 0
      block (result i32)  ;; label = @2
        i32.const 1049384
        i32.load
        local.tee 3
        i32.const 1
        local.get 2
        i32.const 31
        i32.and
        i32.shl
        local.tee 2
        i32.and
        if  ;; label = @3
          local.get 0
          i32.load offset=8
          br 1 (;@2;)
        end
        i32.const 1049384
        local.get 2
        local.get 3
        i32.or
        i32.store
        local.get 0
      end
      local.set 2
      local.get 0
      local.get 1
      i32.store offset=8
      local.get 2
      local.get 1
      i32.store offset=12
      local.get 1
      local.get 0
      i32.store offset=12
      local.get 1
      local.get 2
      i32.store offset=8
      return
    end
    i32.const 1049816
    i32.load
    local.tee 0
    i32.eqz
    if  ;; label = @1
      i32.const 1049832
      i32.const 4095
      i32.store
      return
    end
    i32.const 0
    local.set 1
    loop  ;; label = @1
      local.get 1
      i32.const 1
      i32.add
      local.set 1
      local.get 0
      i32.load offset=8
      local.tee 0
      br_if 0 (;@1;)
    end
    i32.const 1049832
    local.get 1
    i32.const 4095
    local.get 1
    i32.const 4095
    i32.gt_u
    select
    i32.store)
  (func (;2;) (type 2) (param i32 i32)
    (local i32 i32 i32 i32)
    local.get 0
    local.get 1
    i32.add
    local.set 2
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              i32.const 4
              i32.add
              i32.load
              local.tee 3
              i32.const 1
              i32.and
              br_if 0 (;@5;)
              local.get 3
              i32.const 3
              i32.and
              i32.eqz
              br_if 1 (;@4;)
              local.get 0
              i32.load
              local.tee 3
              local.get 1
              i32.add
              local.set 1
              local.get 0
              local.get 3
              i32.sub
              local.tee 0
              i32.const 1049792
              i32.load
              i32.eq
              if  ;; label = @6
                local.get 2
                i32.load offset=4
                i32.const 3
                i32.and
                i32.const 3
                i32.ne
                br_if 1 (;@5;)
                i32.const 1049784
                local.get 1
                i32.store
                local.get 2
                local.get 2
                i32.load offset=4
                i32.const -2
                i32.and
                i32.store offset=4
                local.get 0
                local.get 1
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 2
                local.get 1
                i32.store
                return
              end
              local.get 0
              local.get 3
              call 8
            end
            block  ;; label = @5
              local.get 2
              i32.const 4
              i32.add
              i32.load
              local.tee 3
              i32.const 2
              i32.and
              if  ;; label = @6
                local.get 2
                i32.const 4
                i32.add
                local.get 3
                i32.const -2
                i32.and
                i32.store
                local.get 0
                local.get 1
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 0
                local.get 1
                i32.add
                local.get 1
                i32.store
                br 1 (;@5;)
              end
              block  ;; label = @6
                local.get 2
                i32.const 1049796
                i32.load
                i32.ne
                if  ;; label = @7
                  i32.const 1049792
                  i32.load
                  local.get 2
                  i32.eq
                  br_if 1 (;@6;)
                  local.get 2
                  local.get 3
                  i32.const -8
                  i32.and
                  local.tee 2
                  call 8
                  local.get 0
                  local.get 1
                  local.get 2
                  i32.add
                  local.tee 1
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  local.get 0
                  local.get 1
                  i32.add
                  local.get 1
                  i32.store
                  local.get 0
                  i32.const 1049792
                  i32.load
                  i32.ne
                  br_if 2 (;@5;)
                  i32.const 1049784
                  local.get 1
                  i32.store
                  return
                end
                i32.const 1049796
                local.get 0
                i32.store
                i32.const 1049788
                i32.const 1049788
                i32.load
                local.get 1
                i32.add
                local.tee 1
                i32.store
                local.get 0
                local.get 1
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 0
                i32.const 1049792
                i32.load
                i32.ne
                br_if 2 (;@4;)
                i32.const 1049784
                i32.const 0
                i32.store
                i32.const 1049792
                i32.const 0
                i32.store
                return
              end
              i32.const 1049792
              local.get 0
              i32.store
              i32.const 1049784
              i32.const 1049784
              i32.load
              local.get 1
              i32.add
              local.tee 1
              i32.store
              local.get 0
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 0
              local.get 1
              i32.add
              local.get 1
              i32.store
              return
            end
            local.get 1
            i32.const 256
            i32.lt_u
            br_if 3 (;@1;)
            local.get 0
            i64.const 0
            i64.store offset=16 align=4
            local.get 0
            i32.const 28
            i32.add
            block (result i32)  ;; label = @5
              i32.const 0
              local.get 1
              i32.const 8
              i32.shr_u
              local.tee 2
              i32.eqz
              br_if 0 (;@5;)
              drop
              i32.const 31
              local.get 1
              i32.const 16777215
              i32.gt_u
              br_if 0 (;@5;)
              drop
              local.get 1
              i32.const 6
              local.get 2
              i32.clz
              local.tee 2
              i32.sub
              i32.const 31
              i32.and
              i32.shr_u
              i32.const 1
              i32.and
              local.get 2
              i32.const 1
              i32.shl
              i32.sub
              i32.const 62
              i32.add
            end
            local.tee 3
            i32.store
            local.get 3
            i32.const 2
            i32.shl
            i32.const 1049656
            i32.add
            local.set 2
            block  ;; label = @5
              block  ;; label = @6
                i32.const 1049388
                i32.load
                local.tee 4
                i32.const 1
                local.get 3
                i32.const 31
                i32.and
                i32.shl
                local.tee 5
                i32.and
                if  ;; label = @7
                  local.get 2
                  i32.load
                  local.tee 2
                  i32.const 4
                  i32.add
                  i32.load
                  i32.const -8
                  i32.and
                  local.get 1
                  i32.ne
                  br_if 1 (;@6;)
                  local.get 2
                  local.set 3
                  br 2 (;@5;)
                end
                i32.const 1049388
                local.get 4
                local.get 5
                i32.or
                i32.store
                local.get 2
                local.get 0
                i32.store
                br 4 (;@2;)
              end
              local.get 1
              i32.const 0
              i32.const 25
              local.get 3
              i32.const 1
              i32.shr_u
              i32.sub
              i32.const 31
              i32.and
              local.get 3
              i32.const 31
              i32.eq
              select
              i32.shl
              local.set 4
              loop  ;; label = @6
                local.get 2
                local.get 4
                i32.const 29
                i32.shr_u
                i32.const 4
                i32.and
                i32.add
                i32.const 16
                i32.add
                local.tee 5
                i32.load
                local.tee 3
                i32.eqz
                br_if 3 (;@3;)
                local.get 4
                i32.const 1
                i32.shl
                local.set 4
                local.get 3
                local.tee 2
                i32.const 4
                i32.add
                i32.load
                i32.const -8
                i32.and
                local.get 1
                i32.ne
                br_if 0 (;@6;)
              end
            end
            local.get 3
            i32.load offset=8
            local.tee 1
            local.get 0
            i32.store offset=12
            local.get 3
            local.get 0
            i32.store offset=8
            local.get 0
            i32.const 24
            i32.add
            i32.const 0
            i32.store
            local.get 0
            local.get 3
            i32.store offset=12
            local.get 0
            local.get 1
            i32.store offset=8
          end
          return
        end
        local.get 5
        local.get 0
        i32.store
      end
      local.get 0
      i32.const 24
      i32.add
      local.get 2
      i32.store
      local.get 0
      local.get 0
      i32.store offset=12
      local.get 0
      local.get 0
      i32.store offset=8
      return
    end
    local.get 1
    i32.const 3
    i32.shr_u
    local.tee 2
    i32.const 3
    i32.shl
    i32.const 1049392
    i32.add
    local.set 1
    block (result i32)  ;; label = @1
      i32.const 1049384
      i32.load
      local.tee 3
      i32.const 1
      local.get 2
      i32.const 31
      i32.and
      i32.shl
      local.tee 2
      i32.and
      if  ;; label = @2
        local.get 1
        i32.load offset=8
        br 1 (;@1;)
      end
      i32.const 1049384
      local.get 2
      local.get 3
      i32.or
      i32.store
      local.get 1
    end
    local.set 2
    local.get 1
    local.get 0
    i32.store offset=8
    local.get 2
    local.get 0
    i32.store offset=12
    local.get 0
    local.get 1
    i32.store offset=12
    local.get 0
    local.get 2
    i32.store offset=8)
  (func (;3;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    i32.const 36
    i32.add
    i32.const 1048688
    i32.store
    local.get 2
    i32.const 3
    i32.store8 offset=40
    local.get 2
    i64.const 137438953472
    i64.store offset=8
    local.get 2
    local.get 0
    i32.store offset=32
    local.get 2
    i32.const 0
    i32.store offset=24
    local.get 2
    i32.const 0
    i32.store offset=16
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          local.get 1
          i32.load offset=8
          local.tee 4
          if  ;; label = @4
            local.get 1
            i32.load
            local.set 6
            local.get 1
            i32.load offset=4
            local.tee 9
            local.get 1
            i32.const 12
            i32.add
            i32.load
            local.tee 3
            local.get 3
            local.get 9
            i32.gt_u
            select
            local.tee 11
            i32.eqz
            br_if 1 (;@3;)
            local.get 1
            i32.const 20
            i32.add
            i32.load
            local.set 7
            local.get 1
            i32.load offset=16
            local.set 8
            i32.const 1
            local.set 3
            local.get 0
            local.get 6
            i32.load
            local.get 6
            i32.load offset=4
            i32.const 1048700
            i32.load
            call_indirect (type 1)
            br_if 3 (;@1;)
            local.get 6
            i32.const 12
            i32.add
            local.set 1
            i32.const 1
            local.set 5
            block  ;; label = @5
              block  ;; label = @6
                loop  ;; label = @7
                  local.get 2
                  local.get 4
                  i32.const 4
                  i32.add
                  i32.load
                  i32.store offset=12
                  local.get 2
                  local.get 4
                  i32.const 28
                  i32.add
                  i32.load8_u
                  i32.store8 offset=40
                  local.get 2
                  local.get 4
                  i32.const 8
                  i32.add
                  i32.load
                  i32.store offset=8
                  local.get 4
                  i32.const 24
                  i32.add
                  i32.load
                  local.set 3
                  i32.const 0
                  local.set 10
                  i32.const 0
                  local.set 0
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 4
                        i32.const 20
                        i32.add
                        i32.load
                        i32.const 1
                        i32.sub
                        br_table 0 (;@10;) 2 (;@8;) 1 (;@9;)
                      end
                      local.get 3
                      local.get 7
                      i32.ge_u
                      br_if 3 (;@6;)
                      local.get 3
                      i32.const 3
                      i32.shl
                      local.get 8
                      i32.add
                      local.tee 12
                      i32.load offset=4
                      i32.const 13
                      i32.ne
                      br_if 1 (;@8;)
                      local.get 12
                      i32.load
                      i32.load
                      local.set 3
                    end
                    i32.const 1
                    local.set 0
                  end
                  local.get 2
                  local.get 3
                  i32.store offset=20
                  local.get 2
                  local.get 0
                  i32.store offset=16
                  local.get 4
                  i32.const 16
                  i32.add
                  i32.load
                  local.set 3
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        local.get 4
                        i32.const 12
                        i32.add
                        i32.load
                        i32.const 1
                        i32.sub
                        br_table 0 (;@10;) 2 (;@8;) 1 (;@9;)
                      end
                      local.get 3
                      local.get 7
                      i32.ge_u
                      br_if 4 (;@5;)
                      local.get 3
                      i32.const 3
                      i32.shl
                      local.get 8
                      i32.add
                      local.tee 0
                      i32.load offset=4
                      i32.const 13
                      i32.ne
                      br_if 1 (;@8;)
                      local.get 0
                      i32.load
                      i32.load
                      local.set 3
                    end
                    i32.const 1
                    local.set 10
                  end
                  local.get 2
                  local.get 3
                  i32.store offset=28
                  local.get 2
                  local.get 10
                  i32.store offset=24
                  local.get 4
                  i32.load
                  local.tee 0
                  local.get 7
                  i32.lt_u
                  if  ;; label = @8
                    local.get 8
                    local.get 0
                    i32.const 3
                    i32.shl
                    i32.add
                    local.tee 0
                    i32.load
                    local.get 2
                    i32.const 8
                    i32.add
                    local.get 0
                    i32.load offset=4
                    call_indirect (type 0)
                    br_if 6 (;@2;)
                    local.get 5
                    local.get 11
                    i32.ge_u
                    br_if 5 (;@3;)
                    local.get 1
                    i32.const -4
                    i32.add
                    local.set 0
                    local.get 1
                    i32.load
                    local.set 10
                    local.get 1
                    i32.const 8
                    i32.add
                    local.set 1
                    local.get 4
                    i32.const 32
                    i32.add
                    local.set 4
                    i32.const 1
                    local.set 3
                    local.get 5
                    i32.const 1
                    i32.add
                    local.set 5
                    local.get 2
                    i32.load offset=32
                    local.get 0
                    i32.load
                    local.get 10
                    local.get 2
                    i32.load offset=36
                    i32.load offset=12
                    call_indirect (type 1)
                    i32.eqz
                    br_if 1 (;@7;)
                    br 7 (;@1;)
                  end
                end
                local.get 0
                local.get 7
                i32.const 1049300
                call 15
                unreachable
              end
              local.get 3
              local.get 7
              i32.const 1049316
              call 15
              unreachable
            end
            local.get 3
            local.get 7
            i32.const 1049316
            call 15
            unreachable
          end
          local.get 1
          i32.load
          local.set 6
          local.get 1
          i32.load offset=4
          local.tee 9
          local.get 1
          i32.const 20
          i32.add
          i32.load
          local.tee 3
          local.get 3
          local.get 9
          i32.gt_u
          select
          local.tee 7
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          i32.load offset=16
          local.set 4
          i32.const 1
          local.set 3
          local.get 0
          local.get 6
          i32.load
          local.get 6
          i32.load offset=4
          i32.const 1048700
          i32.load
          call_indirect (type 1)
          br_if 2 (;@1;)
          local.get 6
          i32.const 12
          i32.add
          local.set 1
          i32.const 1
          local.set 5
          loop  ;; label = @4
            local.get 4
            i32.load
            local.get 2
            i32.const 8
            i32.add
            local.get 4
            i32.const 4
            i32.add
            i32.load
            call_indirect (type 0)
            br_if 2 (;@2;)
            local.get 5
            local.get 7
            i32.ge_u
            br_if 1 (;@3;)
            local.get 1
            i32.const -4
            i32.add
            local.set 0
            local.get 1
            i32.load
            local.set 8
            local.get 1
            i32.const 8
            i32.add
            local.set 1
            local.get 4
            i32.const 8
            i32.add
            local.set 4
            local.get 5
            i32.const 1
            i32.add
            local.set 5
            local.get 2
            i32.load offset=32
            local.get 0
            i32.load
            local.get 8
            local.get 2
            i32.load offset=36
            i32.load offset=12
            call_indirect (type 1)
            i32.eqz
            br_if 0 (;@4;)
          end
          br 2 (;@1;)
        end
        local.get 9
        local.get 5
        i32.gt_u
        if  ;; label = @3
          i32.const 1
          local.set 3
          local.get 2
          i32.load offset=32
          local.get 6
          local.get 5
          i32.const 3
          i32.shl
          i32.add
          local.tee 0
          i32.load
          local.get 0
          i32.load offset=4
          local.get 2
          i32.load offset=36
          i32.load offset=12
          call_indirect (type 1)
          br_if 2 (;@1;)
        end
        i32.const 0
        local.set 3
        br 1 (;@1;)
      end
      i32.const 1
      local.set 3
    end
    local.get 2
    i32.const 48
    i32.add
    global.set 0
    local.get 3)
  (func (;4;) (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    i32.const 43
    i32.const 1114112
    local.get 0
    i32.load
    local.tee 5
    i32.const 1
    i32.and
    local.tee 3
    select
    local.set 6
    local.get 2
    local.get 3
    i32.add
    local.set 4
    i32.const 1048992
    i32.const 0
    local.get 5
    i32.const 4
    i32.and
    select
    local.set 7
    i32.const 1
    local.set 3
    block  ;; label = @1
      local.get 0
      i32.load offset=8
      i32.const 1
      i32.ne
      if  ;; label = @2
        local.get 0
        local.get 6
        local.get 7
        call 18
        br_if 1 (;@1;)
        local.get 0
        i32.load offset=24
        local.get 1
        local.get 2
        local.get 0
        i32.const 28
        i32.add
        i32.load
        i32.load offset=12
        call_indirect (type 1)
        local.set 3
        br 1 (;@1;)
      end
      local.get 0
      i32.const 12
      i32.add
      i32.load
      local.tee 8
      local.get 4
      i32.le_u
      if  ;; label = @2
        local.get 0
        local.get 6
        local.get 7
        call 18
        br_if 1 (;@1;)
        local.get 0
        i32.load offset=24
        local.get 1
        local.get 2
        local.get 0
        i32.const 28
        i32.add
        i32.load
        i32.load offset=12
        call_indirect (type 1)
        return
      end
      block  ;; label = @2
        local.get 5
        i32.const 8
        i32.and
        i32.eqz
        if  ;; label = @3
          i32.const 0
          local.set 3
          local.get 8
          local.get 4
          i32.sub
          local.tee 4
          local.set 5
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                i32.const 1
                local.get 0
                i32.load8_u offset=32
                local.tee 8
                local.get 8
                i32.const 3
                i32.eq
                select
                i32.const 1
                i32.sub
                br_table 1 (;@5;) 0 (;@6;) 1 (;@5;) 2 (;@4;)
              end
              local.get 4
              i32.const 1
              i32.shr_u
              local.set 3
              local.get 4
              i32.const 1
              i32.add
              i32.const 1
              i32.shr_u
              local.set 5
              br 1 (;@4;)
            end
            i32.const 0
            local.set 5
            local.get 4
            local.set 3
          end
          local.get 3
          i32.const 1
          i32.add
          local.set 3
          loop  ;; label = @4
            local.get 3
            i32.const -1
            i32.add
            local.tee 3
            i32.eqz
            br_if 2 (;@2;)
            local.get 0
            i32.load offset=24
            local.get 0
            i32.load offset=4
            local.get 0
            i32.load offset=28
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 0 (;@4;)
          end
          i32.const 1
          return
        end
        local.get 0
        i32.load offset=4
        local.set 9
        local.get 0
        i32.const 48
        i32.store offset=4
        local.get 0
        i32.load8_u offset=32
        local.set 10
        local.get 0
        i32.const 1
        i32.store8 offset=32
        local.get 0
        local.get 6
        local.get 7
        call 18
        br_if 1 (;@1;)
        i32.const 0
        local.set 3
        local.get 8
        local.get 4
        i32.sub
        local.tee 4
        local.set 5
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              i32.const 1
              local.get 0
              i32.load8_u offset=32
              local.tee 6
              local.get 6
              i32.const 3
              i32.eq
              select
              i32.const 1
              i32.sub
              br_table 1 (;@4;) 0 (;@5;) 1 (;@4;) 2 (;@3;)
            end
            local.get 4
            i32.const 1
            i32.shr_u
            local.set 3
            local.get 4
            i32.const 1
            i32.add
            i32.const 1
            i32.shr_u
            local.set 5
            br 1 (;@3;)
          end
          i32.const 0
          local.set 5
          local.get 4
          local.set 3
        end
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        block  ;; label = @3
          loop  ;; label = @4
            local.get 3
            i32.const -1
            i32.add
            local.tee 3
            i32.eqz
            br_if 1 (;@3;)
            local.get 0
            i32.load offset=24
            local.get 0
            i32.load offset=4
            local.get 0
            i32.load offset=28
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 0 (;@4;)
          end
          i32.const 1
          return
        end
        local.get 0
        i32.load offset=4
        local.set 4
        i32.const 1
        local.set 3
        local.get 0
        i32.load offset=24
        local.get 1
        local.get 2
        local.get 0
        i32.load offset=28
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
        local.get 5
        i32.const 1
        i32.add
        local.set 1
        local.get 0
        i32.load offset=28
        local.set 2
        local.get 0
        i32.load offset=24
        local.set 5
        loop  ;; label = @3
          local.get 1
          i32.const -1
          i32.add
          local.tee 1
          if  ;; label = @4
            local.get 5
            local.get 4
            local.get 2
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 1 (;@3;)
            br 3 (;@1;)
          end
        end
        local.get 0
        local.get 10
        i32.store8 offset=32
        local.get 0
        local.get 9
        i32.store offset=4
        i32.const 0
        return
      end
      local.get 0
      i32.load offset=4
      local.set 4
      i32.const 1
      local.set 3
      local.get 0
      local.get 6
      local.get 7
      call 18
      br_if 0 (;@1;)
      local.get 0
      i32.load offset=24
      local.get 1
      local.get 2
      local.get 0
      i32.load offset=28
      i32.load offset=12
      call_indirect (type 1)
      br_if 0 (;@1;)
      local.get 5
      i32.const 1
      i32.add
      local.set 1
      local.get 0
      i32.load offset=28
      local.set 2
      local.get 0
      i32.load offset=24
      local.set 0
      loop  ;; label = @2
        local.get 1
        i32.const -1
        i32.add
        local.tee 1
        i32.eqz
        if  ;; label = @3
          i32.const 0
          return
        end
        local.get 0
        local.get 4
        local.get 2
        i32.load offset=16
        call_indirect (type 0)
        i32.eqz
        br_if 0 (;@2;)
      end
    end
    local.get 3)
  (func (;5;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      local.get 1
      i32.const -65588
      i32.gt_u
      br_if 0 (;@1;)
      i32.const 16
      local.get 1
      i32.const 11
      i32.add
      i32.const -8
      i32.and
      local.get 1
      i32.const 11
      i32.lt_u
      select
      local.set 2
      local.get 0
      i32.const -4
      i32.add
      local.tee 5
      i32.load
      local.tee 6
      i32.const -8
      i32.and
      local.set 3
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  local.get 6
                  i32.const 3
                  i32.and
                  if  ;; label = @8
                    local.get 0
                    i32.const -8
                    i32.add
                    local.tee 7
                    local.get 3
                    i32.add
                    local.set 8
                    local.get 3
                    local.get 2
                    i32.ge_u
                    br_if 1 (;@7;)
                    i32.const 1049796
                    i32.load
                    local.get 8
                    i32.eq
                    br_if 2 (;@6;)
                    i32.const 1049792
                    i32.load
                    local.get 8
                    i32.eq
                    br_if 3 (;@5;)
                    local.get 8
                    i32.const 4
                    i32.add
                    i32.load
                    local.tee 6
                    i32.const 2
                    i32.and
                    br_if 6 (;@2;)
                    local.get 6
                    i32.const -8
                    i32.and
                    local.tee 6
                    local.get 3
                    i32.add
                    local.tee 3
                    local.get 2
                    i32.ge_u
                    br_if 4 (;@4;)
                    br 6 (;@2;)
                  end
                  local.get 2
                  i32.const 256
                  i32.lt_u
                  local.get 3
                  local.get 2
                  i32.const 4
                  i32.or
                  i32.lt_u
                  i32.or
                  local.get 3
                  local.get 2
                  i32.sub
                  i32.const 131073
                  i32.ge_u
                  i32.or
                  br_if 5 (;@2;)
                  br 4 (;@3;)
                end
                local.get 3
                local.get 2
                i32.sub
                local.tee 1
                i32.const 16
                i32.lt_u
                br_if 3 (;@3;)
                local.get 5
                local.get 2
                local.get 6
                i32.const 1
                i32.and
                i32.or
                i32.const 2
                i32.or
                i32.store
                local.get 2
                local.get 7
                i32.add
                local.tee 4
                local.get 1
                i32.const 3
                i32.or
                i32.store offset=4
                local.get 8
                local.get 8
                i32.load offset=4
                i32.const 1
                i32.or
                i32.store offset=4
                local.get 4
                local.get 1
                call 2
                br 3 (;@3;)
              end
              i32.const 1049788
              i32.load
              local.get 3
              i32.add
              local.tee 3
              local.get 2
              i32.le_u
              br_if 3 (;@2;)
              local.get 5
              local.get 2
              local.get 6
              i32.const 1
              i32.and
              i32.or
              i32.const 2
              i32.or
              i32.store
              local.get 2
              local.get 7
              i32.add
              local.tee 1
              local.get 3
              local.get 2
              i32.sub
              local.tee 4
              i32.const 1
              i32.or
              i32.store offset=4
              i32.const 1049788
              local.get 4
              i32.store
              i32.const 1049796
              local.get 1
              i32.store
              br 2 (;@3;)
            end
            i32.const 1049784
            i32.load
            local.get 3
            i32.add
            local.tee 3
            local.get 2
            i32.lt_u
            br_if 2 (;@2;)
            block  ;; label = @5
              local.get 3
              local.get 2
              i32.sub
              local.tee 1
              i32.const 15
              i32.le_u
              if  ;; label = @6
                local.get 5
                local.get 6
                i32.const 1
                i32.and
                local.get 3
                i32.or
                i32.const 2
                i32.or
                i32.store
                local.get 3
                local.get 7
                i32.add
                local.tee 1
                local.get 1
                i32.load offset=4
                i32.const 1
                i32.or
                i32.store offset=4
                i32.const 0
                local.set 1
                br 1 (;@5;)
              end
              local.get 5
              local.get 2
              local.get 6
              i32.const 1
              i32.and
              i32.or
              i32.const 2
              i32.or
              i32.store
              local.get 2
              local.get 7
              i32.add
              local.tee 4
              local.get 1
              i32.const 1
              i32.or
              i32.store offset=4
              local.get 3
              local.get 7
              i32.add
              local.tee 2
              local.get 1
              i32.store
              local.get 2
              local.get 2
              i32.load offset=4
              i32.const -2
              i32.and
              i32.store offset=4
            end
            i32.const 1049792
            local.get 4
            i32.store
            i32.const 1049784
            local.get 1
            i32.store
            br 1 (;@3;)
          end
          local.get 8
          local.get 6
          call 8
          local.get 3
          local.get 2
          i32.sub
          local.tee 1
          i32.const 16
          i32.ge_u
          if  ;; label = @4
            local.get 5
            local.get 2
            local.get 5
            i32.load
            i32.const 1
            i32.and
            i32.or
            i32.const 2
            i32.or
            i32.store
            local.get 2
            local.get 7
            i32.add
            local.tee 4
            local.get 1
            i32.const 3
            i32.or
            i32.store offset=4
            local.get 3
            local.get 7
            i32.add
            local.tee 2
            local.get 2
            i32.load offset=4
            i32.const 1
            i32.or
            i32.store offset=4
            local.get 4
            local.get 1
            call 2
            br 1 (;@3;)
          end
          local.get 5
          local.get 3
          local.get 5
          i32.load
          i32.const 1
          i32.and
          i32.or
          i32.const 2
          i32.or
          i32.store
          local.get 3
          local.get 7
          i32.add
          local.tee 1
          local.get 1
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
        end
        local.get 0
        local.set 4
        br 1 (;@1;)
      end
      local.get 1
      call 0
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 0
      local.get 1
      local.get 5
      i32.load
      local.tee 4
      i32.const -8
      i32.and
      i32.const 4
      i32.const 8
      local.get 4
      i32.const 3
      i32.and
      select
      i32.sub
      local.tee 4
      local.get 4
      local.get 1
      i32.gt_u
      select
      call 22
      local.get 0
      call 1
      return
    end
    local.get 4)
  (func (;6;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 2
    global.set 0
    local.get 0
    i32.load
    local.set 0
    block  ;; label = @1
      block  ;; label = @2
        block (result i32)  ;; label = @3
          block  ;; label = @4
            local.get 1
            i32.const 128
            i32.ge_u
            if  ;; label = @5
              local.get 2
              i32.const 0
              i32.store offset=12
              local.get 1
              i32.const 2048
              i32.lt_u
              br_if 1 (;@4;)
              local.get 2
              i32.const 12
              i32.add
              local.set 3
              local.get 1
              i32.const 65536
              i32.lt_u
              if  ;; label = @6
                local.get 2
                local.get 1
                i32.const 63
                i32.and
                i32.const 128
                i32.or
                i32.store8 offset=14
                local.get 2
                local.get 1
                i32.const 6
                i32.shr_u
                i32.const 63
                i32.and
                i32.const 128
                i32.or
                i32.store8 offset=13
                local.get 2
                local.get 1
                i32.const 12
                i32.shr_u
                i32.const 15
                i32.and
                i32.const 224
                i32.or
                i32.store8 offset=12
                i32.const 3
                br 3 (;@3;)
              end
              local.get 2
              local.get 1
              i32.const 63
              i32.and
              i32.const 128
              i32.or
              i32.store8 offset=15
              local.get 2
              local.get 1
              i32.const 18
              i32.shr_u
              i32.const 240
              i32.or
              i32.store8 offset=12
              local.get 2
              local.get 1
              i32.const 6
              i32.shr_u
              i32.const 63
              i32.and
              i32.const 128
              i32.or
              i32.store8 offset=14
              local.get 2
              local.get 1
              i32.const 12
              i32.shr_u
              i32.const 63
              i32.and
              i32.const 128
              i32.or
              i32.store8 offset=13
              i32.const 4
              br 2 (;@3;)
            end
            local.get 0
            i32.load offset=8
            local.tee 3
            local.get 0
            i32.const 4
            i32.add
            i32.load
            i32.eq
            if (result i32)  ;; label = @5
              local.get 2
              i32.const 16
              i32.add
              local.get 0
              local.get 3
              i32.const 1
              call 10
              local.get 2
              i32.load offset=16
              i32.const 1
              i32.eq
              if  ;; label = @6
                local.get 2
                i32.const 24
                i32.add
                i32.load
                i32.eqz
                br_if 5 (;@1;)
                i32.const 1048771
                i32.const 40
                i32.const 1048828
                call 19
                unreachable
              end
              local.get 0
              i32.load offset=8
            else
              local.get 3
            end
            local.get 0
            i32.load
            i32.add
            local.get 1
            i32.store8
            local.get 0
            local.get 0
            i32.load offset=8
            i32.const 1
            i32.add
            i32.store offset=8
            br 2 (;@2;)
          end
          local.get 2
          local.get 1
          i32.const 63
          i32.and
          i32.const 128
          i32.or
          i32.store8 offset=13
          local.get 2
          local.get 1
          i32.const 6
          i32.shr_u
          i32.const 31
          i32.and
          i32.const 192
          i32.or
          i32.store8 offset=12
          local.get 2
          i32.const 12
          i32.add
          local.set 3
          i32.const 2
        end
        local.set 1
        local.get 2
        i32.const 16
        i32.add
        local.get 0
        local.get 0
        i32.const 8
        i32.add
        local.tee 4
        i32.load
        local.get 1
        call 10
        local.get 2
        i32.load offset=16
        i32.const 1
        i32.eq
        if  ;; label = @3
          local.get 2
          i32.const 24
          i32.add
          i32.load
          i32.eqz
          br_if 2 (;@1;)
          i32.const 1048771
          i32.const 40
          i32.const 1048828
          call 19
          unreachable
        end
        local.get 4
        local.get 4
        i32.load
        local.tee 4
        local.get 1
        i32.add
        i32.store
        local.get 4
        local.get 0
        i32.load
        i32.add
        local.get 3
        local.get 1
        call 22
        drop
      end
      local.get 2
      i32.const 32
      i32.add
      global.set 0
      i32.const 0
      return
    end
    call 27
    unreachable)
  (func (;7;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32)
    block  ;; label = @1
      i32.const -65587
      local.get 0
      i32.const 16
      local.get 0
      i32.const 16
      i32.gt_u
      select
      local.tee 0
      i32.sub
      local.get 1
      i32.le_u
      br_if 0 (;@1;)
      local.get 0
      i32.const 16
      local.get 1
      i32.const 11
      i32.add
      i32.const -8
      i32.and
      local.get 1
      i32.const 11
      i32.lt_u
      select
      local.tee 4
      i32.add
      i32.const 12
      i32.add
      call 0
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      i32.const -8
      i32.add
      local.set 1
      block  ;; label = @2
        local.get 0
        i32.const -1
        i32.add
        local.tee 3
        local.get 2
        i32.and
        i32.eqz
        if  ;; label = @3
          local.get 1
          local.set 0
          br 1 (;@2;)
        end
        local.get 2
        i32.const -4
        i32.add
        local.tee 5
        i32.load
        local.tee 6
        i32.const -8
        i32.and
        local.get 2
        local.get 3
        i32.add
        i32.const 0
        local.get 0
        i32.sub
        i32.and
        i32.const -8
        i32.add
        local.tee 2
        local.get 0
        local.get 2
        i32.add
        local.get 2
        local.get 1
        i32.sub
        i32.const 16
        i32.gt_u
        select
        local.tee 0
        local.get 1
        i32.sub
        local.tee 2
        i32.sub
        local.set 3
        local.get 6
        i32.const 3
        i32.and
        if  ;; label = @3
          local.get 0
          local.get 3
          local.get 0
          i32.load offset=4
          i32.const 1
          i32.and
          i32.or
          i32.const 2
          i32.or
          i32.store offset=4
          local.get 0
          local.get 3
          i32.add
          local.tee 3
          local.get 3
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 5
          local.get 2
          local.get 5
          i32.load
          i32.const 1
          i32.and
          i32.or
          i32.const 2
          i32.or
          i32.store
          local.get 0
          local.get 0
          i32.load offset=4
          i32.const 1
          i32.or
          i32.store offset=4
          local.get 1
          local.get 2
          call 2
          br 1 (;@2;)
        end
        local.get 1
        i32.load
        local.set 1
        local.get 0
        local.get 3
        i32.store offset=4
        local.get 0
        local.get 1
        local.get 2
        i32.add
        i32.store
      end
      block  ;; label = @2
        local.get 0
        i32.const 4
        i32.add
        i32.load
        local.tee 1
        i32.const 3
        i32.and
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.const -8
        i32.and
        local.tee 2
        local.get 4
        i32.const 16
        i32.add
        i32.le_u
        br_if 0 (;@2;)
        local.get 0
        i32.const 4
        i32.add
        local.get 4
        local.get 1
        i32.const 1
        i32.and
        i32.or
        i32.const 2
        i32.or
        i32.store
        local.get 0
        local.get 4
        i32.add
        local.tee 1
        local.get 2
        local.get 4
        i32.sub
        local.tee 4
        i32.const 3
        i32.or
        i32.store offset=4
        local.get 0
        local.get 2
        i32.add
        local.tee 2
        local.get 2
        i32.load offset=4
        i32.const 1
        i32.or
        i32.store offset=4
        local.get 1
        local.get 4
        call 2
      end
      local.get 0
      i32.const 8
      i32.add
      local.set 3
    end
    local.get 3)
  (func (;8;) (type 2) (param i32 i32)
    (local i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        local.get 1
        i32.const 256
        i32.ge_u
        if  ;; label = @3
          local.get 0
          i32.const 24
          i32.add
          i32.load
          local.set 4
          block  ;; label = @4
            block  ;; label = @5
              local.get 0
              local.get 0
              i32.load offset=12
              local.tee 2
              i32.eq
              if  ;; label = @6
                local.get 0
                i32.const 20
                i32.const 16
                local.get 0
                i32.const 20
                i32.add
                local.tee 2
                i32.load
                local.tee 3
                select
                i32.add
                i32.load
                local.tee 1
                br_if 1 (;@5;)
                i32.const 0
                local.set 2
                br 2 (;@4;)
              end
              local.get 0
              i32.load offset=8
              local.tee 1
              local.get 2
              i32.store offset=12
              local.get 2
              local.get 1
              i32.store offset=8
              br 1 (;@4;)
            end
            local.get 2
            local.get 0
            i32.const 16
            i32.add
            local.get 3
            select
            local.set 3
            loop  ;; label = @5
              local.get 3
              local.set 5
              local.get 1
              local.tee 2
              i32.const 20
              i32.add
              local.tee 3
              i32.load
              local.tee 1
              i32.eqz
              if  ;; label = @6
                local.get 2
                i32.const 16
                i32.add
                local.set 3
                local.get 2
                i32.load offset=16
                local.set 1
              end
              local.get 1
              br_if 0 (;@5;)
            end
            local.get 5
            i32.const 0
            i32.store
          end
          local.get 4
          i32.eqz
          br_if 2 (;@1;)
          local.get 0
          local.get 0
          i32.const 28
          i32.add
          i32.load
          i32.const 2
          i32.shl
          i32.const 1049656
          i32.add
          local.tee 1
          i32.load
          i32.ne
          if  ;; label = @4
            local.get 4
            i32.const 16
            i32.const 20
            local.get 4
            i32.load offset=16
            local.get 0
            i32.eq
            select
            i32.add
            local.get 2
            i32.store
            local.get 2
            i32.eqz
            br_if 3 (;@1;)
            br 2 (;@2;)
          end
          local.get 1
          local.get 2
          i32.store
          local.get 2
          br_if 1 (;@2;)
          i32.const 1049388
          i32.const 1049388
          i32.load
          i32.const -2
          local.get 0
          i32.load offset=28
          i32.rotl
          i32.and
          i32.store
          return
        end
        local.get 0
        i32.const 12
        i32.add
        i32.load
        local.tee 2
        local.get 0
        i32.const 8
        i32.add
        i32.load
        local.tee 0
        i32.ne
        if  ;; label = @3
          local.get 0
          local.get 2
          i32.store offset=12
          local.get 2
          local.get 0
          i32.store offset=8
          return
        end
        i32.const 1049384
        i32.const 1049384
        i32.load
        i32.const -2
        local.get 1
        i32.const 3
        i32.shr_u
        i32.rotl
        i32.and
        i32.store
        br 1 (;@1;)
      end
      local.get 2
      local.get 4
      i32.store offset=24
      local.get 0
      i32.load offset=16
      local.tee 1
      if  ;; label = @2
        local.get 2
        local.get 1
        i32.store offset=16
        local.get 1
        local.get 2
        i32.store offset=24
      end
      local.get 0
      i32.const 20
      i32.add
      i32.load
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      i32.const 20
      i32.add
      local.get 0
      i32.store
      local.get 0
      local.get 2
      i32.store offset=24
    end)
  (func (;9;) (type 10) (param i64 i32) (result i32)
    (local i32 i32 i32 i32 i32 i64)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 4
    global.set 0
    i32.const 39
    local.set 2
    block  ;; label = @1
      local.get 0
      i64.const 10000
      i64.lt_u
      if  ;; label = @2
        local.get 0
        local.set 7
        br 1 (;@1;)
      end
      loop  ;; label = @2
        local.get 4
        i32.const 9
        i32.add
        local.get 2
        i32.add
        local.tee 3
        i32.const -4
        i32.add
        local.get 0
        local.get 0
        i64.const 10000
        i64.div_u
        local.tee 7
        i64.const 10000
        i64.mul
        i64.sub
        i32.wrap_i64
        local.tee 5
        i32.const 65535
        i32.and
        i32.const 100
        i32.div_u
        local.tee 6
        i32.const 1
        i32.shl
        i32.const 1049076
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        local.get 3
        i32.const -2
        i32.add
        local.get 5
        local.get 6
        i32.const 100
        i32.mul
        i32.sub
        i32.const 65535
        i32.and
        i32.const 1
        i32.shl
        i32.const 1049076
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        local.get 2
        i32.const -4
        i32.add
        local.set 2
        local.get 0
        i64.const 99999999
        i64.gt_u
        local.get 7
        local.set 0
        br_if 0 (;@2;)
      end
    end
    local.get 7
    i32.wrap_i64
    local.tee 3
    i32.const 99
    i32.gt_s
    if  ;; label = @1
      local.get 2
      i32.const -2
      i32.add
      local.tee 2
      local.get 4
      i32.const 9
      i32.add
      i32.add
      local.get 7
      i32.wrap_i64
      local.tee 3
      local.get 3
      i32.const 65535
      i32.and
      i32.const 100
      i32.div_u
      local.tee 3
      i32.const 100
      i32.mul
      i32.sub
      i32.const 65535
      i32.and
      i32.const 1
      i32.shl
      i32.const 1049076
      i32.add
      i32.load16_u align=1
      i32.store16 align=1
    end
    block  ;; label = @1
      local.get 3
      i32.const 10
      i32.ge_s
      if  ;; label = @2
        local.get 2
        i32.const -2
        i32.add
        local.tee 2
        local.get 4
        i32.const 9
        i32.add
        i32.add
        local.get 3
        i32.const 1
        i32.shl
        i32.const 1049076
        i32.add
        i32.load16_u align=1
        i32.store16 align=1
        br 1 (;@1;)
      end
      local.get 2
      i32.const -1
      i32.add
      local.tee 2
      local.get 4
      i32.const 9
      i32.add
      i32.add
      local.get 3
      i32.const 48
      i32.add
      i32.store8
    end
    local.get 1
    local.get 4
    i32.const 9
    i32.add
    local.get 2
    i32.add
    i32.const 39
    local.get 2
    i32.sub
    call 4
    local.get 4
    i32.const 48
    i32.add
    global.set 0)
  (func (;10;) (type 8) (param i32 i32 i32 i32)
    (local i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 4
    global.set 0
    local.get 0
    block (result i32)  ;; label = @1
      i32.const 0
      local.get 1
      i32.const 4
      i32.add
      i32.load
      local.tee 5
      local.get 2
      i32.sub
      local.get 3
      i32.ge_u
      br_if 0 (;@1;)
      drop
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            local.get 2
            local.get 3
            i32.add
            local.tee 3
            local.get 2
            i32.ge_u
            if  ;; label = @5
              local.get 5
              i32.const 1
              i32.shl
              local.tee 2
              local.get 3
              local.get 2
              local.get 3
              i32.gt_u
              select
              local.set 2
              br 1 (;@4;)
            end
            local.get 0
            local.get 3
            i32.store offset=4
            br 1 (;@3;)
          end
          local.get 2
          i32.const -1
          i32.gt_s
          br_if 1 (;@2;)
        end
        local.get 0
        i32.const 8
        i32.add
        i32.const 0
        i32.store
        i32.const 1
        br 1 (;@1;)
      end
      block (result i32)  ;; label = @2
        local.get 5
        i32.eqz
        if  ;; label = @3
          local.get 4
          i32.const 1
          i32.store offset=12
          local.get 4
          local.get 2
          i32.store offset=8
          local.get 2
          if  ;; label = @4
            local.get 2
            i32.const 1
            call 26
            br 2 (;@2;)
          end
          local.get 4
          i32.const 8
          i32.add
          call 31
          br 1 (;@2;)
        end
        local.get 1
        i32.load
        local.set 0
        local.get 4
        local.get 5
        i32.const 0
        i32.ne
        local.tee 3
        i32.store offset=12
        local.get 4
        local.get 5
        i32.store offset=8
        local.get 2
        if  ;; label = @3
          local.get 0
          local.get 5
          local.get 3
          local.get 2
          call 17
          br 1 (;@2;)
        end
        local.get 0
        call 1
        local.get 4
        i32.const 8
        i32.add
        call 31
      end
      local.tee 0
      i32.eqz
      if  ;; label = @2
        local.get 2
        i32.const 1
        call 30
        unreachable
      end
      local.get 1
      local.get 0
      i32.store
      local.get 1
      i32.const 4
      i32.add
      local.get 2
      i32.store
      i32.const 0
    end
    i32.store
    local.get 4
    i32.const 16
    i32.add
    global.set 0)
  (func (;11;) (type 2) (param i32 i32)
    (local i32 i32 i32 i32)
    global.get 0
    i32.const -64
    i32.add
    local.tee 2
    global.set 0
    local.get 1
    i32.load offset=4
    local.tee 3
    i32.eqz
    if  ;; label = @1
      local.get 1
      i32.const 4
      i32.add
      local.set 3
      local.get 1
      i32.load
      local.set 4
      local.get 2
      i32.const 0
      i32.store offset=32
      local.get 2
      i64.const 1
      i64.store offset=24
      local.get 2
      local.get 2
      i32.const 24
      i32.add
      i32.store offset=36
      local.get 2
      i32.const 56
      i32.add
      local.get 4
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      i32.const 48
      i32.add
      local.get 4
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      local.get 4
      i64.load align=4
      i64.store offset=40
      local.get 2
      i32.const 36
      i32.add
      local.get 2
      i32.const 40
      i32.add
      call 3
      drop
      local.get 2
      i32.const 16
      i32.add
      local.tee 4
      local.get 2
      i32.load offset=32
      i32.store
      local.get 2
      local.get 2
      i64.load offset=24
      i64.store offset=8
      block  ;; label = @2
        local.get 1
        i32.load offset=4
        local.tee 5
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.const 8
        i32.add
        i32.load
        i32.eqz
        br_if 0 (;@2;)
        local.get 5
        call 1
      end
      local.get 3
      local.get 2
      i64.load offset=8
      i64.store align=4
      local.get 3
      i32.const 8
      i32.add
      local.get 4
      i32.load
      i32.store
      local.get 3
      i32.load
      local.set 3
    end
    local.get 1
    i32.const 1
    i32.store offset=4
    local.get 1
    i32.const 12
    i32.add
    i32.load
    local.set 4
    local.get 1
    i32.const 8
    i32.add
    local.tee 1
    i32.load
    local.set 5
    local.get 1
    i64.const 0
    i64.store align=4
    i32.const 12
    i32.const 4
    call 26
    local.tee 1
    i32.eqz
    if  ;; label = @1
      i32.const 12
      i32.const 4
      call 30
      unreachable
    end
    local.get 1
    local.get 4
    i32.store offset=8
    local.get 1
    local.get 5
    i32.store offset=4
    local.get 1
    local.get 3
    i32.store
    local.get 0
    i32.const 1048920
    i32.store offset=4
    local.get 0
    local.get 1
    i32.store
    local.get 2
    i32.const -64
    i32.sub
    global.set 0)
  (func (;12;) (type 2) (param i32 i32)
    (local i32 i32 i32 i32)
    global.get 0
    i32.const -64
    i32.add
    local.tee 2
    global.set 0
    local.get 1
    i32.const 4
    i32.add
    local.set 4
    local.get 1
    i32.load offset=4
    i32.eqz
    if  ;; label = @1
      local.get 1
      i32.load
      local.set 3
      local.get 2
      i32.const 0
      i32.store offset=32
      local.get 2
      i64.const 1
      i64.store offset=24
      local.get 2
      local.get 2
      i32.const 24
      i32.add
      i32.store offset=36
      local.get 2
      i32.const 56
      i32.add
      local.get 3
      i32.const 16
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      i32.const 48
      i32.add
      local.get 3
      i32.const 8
      i32.add
      i64.load align=4
      i64.store
      local.get 2
      local.get 3
      i64.load align=4
      i64.store offset=40
      local.get 2
      i32.const 36
      i32.add
      local.get 2
      i32.const 40
      i32.add
      call 3
      drop
      local.get 2
      i32.const 16
      i32.add
      local.tee 3
      local.get 2
      i32.load offset=32
      i32.store
      local.get 2
      local.get 2
      i64.load offset=24
      i64.store offset=8
      block  ;; label = @2
        local.get 1
        i32.load offset=4
        local.tee 5
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        i32.const 8
        i32.add
        i32.load
        i32.eqz
        br_if 0 (;@2;)
        local.get 5
        call 1
      end
      local.get 4
      local.get 2
      i64.load offset=8
      i64.store align=4
      local.get 4
      i32.const 8
      i32.add
      local.get 3
      i32.load
      i32.store
    end
    local.get 0
    i32.const 1048920
    i32.store offset=4
    local.get 0
    local.get 4
    i32.store
    local.get 2
    i32.const -64
    i32.sub
    global.set 0)
  (func (;13;) (type 4) (param i32 i32 i32)
    (local i32 i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 3
    global.set 0
    i32.const 1
    local.set 4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          i32.const 1049840
          i32.load
          i32.const 1
          i32.ne
          if  ;; label = @4
            i32.const 1049840
            i64.const 4294967297
            i64.store
            br 1 (;@3;)
          end
          i32.const 1049844
          i32.const 1049844
          i32.load
          i32.const 1
          i32.add
          local.tee 4
          i32.store
          local.get 4
          i32.const 2
          i32.gt_u
          br_if 1 (;@2;)
        end
        local.get 3
        local.get 2
        i32.store offset=28
        local.get 3
        local.get 1
        i32.store offset=24
        local.get 3
        i32.const 1048712
        i32.store offset=20
        local.get 3
        i32.const 1048712
        i32.store offset=16
        i32.const 1049372
        i32.load
        local.tee 1
        i32.const -1
        i32.le_s
        br_if 0 (;@2;)
        i32.const 1049372
        local.get 1
        i32.const 1
        i32.add
        local.tee 1
        i32.store
        i32.const 1049372
        i32.const 1049380
        i32.load
        local.tee 2
        if (result i32)  ;; label = @3
          i32.const 1049376
          i32.load
          local.get 3
          i32.const 8
          i32.add
          local.get 0
          i32.const 1048916
          i32.load
          call_indirect (type 2)
          local.get 3
          local.get 3
          i64.load offset=8
          i64.store offset=16
          local.get 3
          i32.const 16
          i32.add
          local.get 2
          i32.load offset=12
          call_indirect (type 2)
          i32.const 1049372
          i32.load
        else
          local.get 1
        end
        i32.const -1
        i32.add
        i32.store
        local.get 4
        i32.const 1
        i32.le_u
        br_if 1 (;@1;)
      end
      unreachable
    end
    global.get 0
    i32.const 16
    i32.sub
    local.tee 1
    global.set 0
    local.get 1
    i32.const 1048900
    i32.store offset=12
    local.get 1
    local.get 0
    i32.store offset=8
    unreachable)
  (func (;14;) (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    local.get 0
    i32.load
    local.tee 0
    local.get 0
    i32.const 8
    i32.add
    local.tee 4
    i32.load
    local.get 2
    call 10
    block  ;; label = @1
      local.get 3
      i32.load
      i32.const 1
      i32.eq
      if  ;; label = @2
        local.get 3
        i32.const 8
        i32.add
        i32.load
        i32.eqz
        br_if 1 (;@1;)
        i32.const 1048771
        i32.const 40
        i32.const 1048828
        call 19
        unreachable
      end
      local.get 4
      local.get 4
      i32.load
      local.tee 4
      local.get 2
      i32.add
      i32.store
      local.get 4
      local.get 0
      i32.load
      i32.add
      local.get 1
      local.get 2
      call 22
      drop
      local.get 3
      i32.const 16
      i32.add
      global.set 0
      i32.const 0
      return
    end
    call 27
    unreachable)
  (func (;15;) (type 4) (param i32 i32 i32)
    (local i32)
    global.get 0
    i32.const 48
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 28
    i32.add
    i32.const 2
    i32.store
    local.get 3
    i32.const 44
    i32.add
    i32.const 12
    i32.store
    local.get 3
    i64.const 2
    i64.store offset=12 align=4
    local.get 3
    i32.const 1049060
    i32.store offset=8
    local.get 3
    i32.const 12
    i32.store offset=36
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=24
    local.get 3
    local.get 3
    i32.store offset=40
    local.get 3
    local.get 3
    i32.const 4
    i32.add
    i32.store offset=32
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call 23
    unreachable)
  (func (;16;) (type 0) (param i32 i32) (result i32)
    (local i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    local.get 0
    i32.load
    i32.store offset=4
    local.get 2
    i32.const 24
    i32.add
    local.get 1
    i32.const 16
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    i32.const 16
    i32.add
    local.get 1
    i32.const 8
    i32.add
    i64.load align=4
    i64.store
    local.get 2
    local.get 1
    i64.load align=4
    i64.store offset=8
    local.get 2
    i32.const 4
    i32.add
    local.get 2
    i32.const 8
    i32.add
    call 3
    local.get 2
    i32.const 32
    i32.add
    global.set 0)
  (func (;17;) (type 9) (param i32 i32 i32 i32) (result i32)
    block  ;; label = @1
      i32.const 8
      local.get 2
      i32.lt_u
      if  ;; label = @2
        block (result i32)  ;; label = @3
          i32.const 8
          local.get 2
          i32.lt_u
          if  ;; label = @4
            local.get 2
            local.get 3
            call 7
            br 1 (;@3;)
          end
          local.get 3
          call 0
        end
        local.tee 2
        br_if 1 (;@1;)
        i32.const 0
        return
      end
      local.get 0
      local.get 3
      call 5
      return
    end
    local.get 2
    local.get 0
    local.get 3
    local.get 1
    local.get 1
    local.get 3
    i32.gt_u
    select
    call 22
    local.get 0
    call 1)
  (func (;18;) (type 1) (param i32 i32 i32) (result i32)
    block (result i32)  ;; label = @1
      local.get 1
      i32.const 1114112
      i32.ne
      if  ;; label = @2
        i32.const 1
        local.get 0
        i32.load offset=24
        local.get 1
        local.get 0
        i32.const 28
        i32.add
        i32.load
        i32.load offset=16
        call_indirect (type 0)
        br_if 1 (;@1;)
        drop
      end
      local.get 2
      i32.eqz
      if  ;; label = @2
        i32.const 0
        return
      end
      local.get 0
      i32.load offset=24
      local.get 2
      i32.const 0
      local.get 0
      i32.const 28
      i32.add
      i32.load
      i32.load offset=12
      call_indirect (type 1)
    end)
  (func (;19;) (type 4) (param i32 i32 i32)
    (local i32)
    global.get 0
    i32.const 32
    i32.sub
    local.tee 3
    global.set 0
    local.get 3
    i32.const 20
    i32.add
    i32.const 0
    i32.store
    local.get 3
    i32.const 1048992
    i32.store offset=16
    local.get 3
    i64.const 1
    i64.store offset=4 align=4
    local.get 3
    local.get 1
    i32.store offset=28
    local.get 3
    local.get 0
    i32.store offset=24
    local.get 3
    local.get 3
    i32.const 24
    i32.add
    i32.store
    local.get 3
    local.get 2
    call 23
    unreachable)
  (func (;20;) (type 0) (param i32 i32) (result i32)
    block  ;; label = @1
      local.get 1
      if  ;; label = @2
        local.get 0
        i32.const -2147483648
        i32.eq
        i32.const 0
        local.get 1
        i32.const -1
        i32.eq
        select
        br_if 1 (;@1;)
        local.get 0
        local.get 1
        i32.div_s
        return
      end
      i32.const 1048624
      i32.const 25
      i32.const 1048596
      call 19
      unreachable
    end
    i32.const 1048656
    i32.const 31
    i32.const 1048596
    call 19
    unreachable)
  (func (;21;) (type 3) (param i32)
    (local i32 i32 i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 1
    global.set 0
    local.get 0
    i32.load offset=12
    local.tee 2
    i32.eqz
    if  ;; label = @1
      i32.const 1048728
      i32.const 43
      i32.const 1048868
      call 19
      unreachable
    end
    local.get 0
    i32.load offset=8
    local.tee 3
    i32.eqz
    if  ;; label = @1
      i32.const 1048728
      i32.const 43
      i32.const 1048884
      call 19
      unreachable
    end
    local.get 1
    i32.const 0
    i32.store offset=4
    local.get 1
    local.get 3
    i32.store
    local.get 1
    local.get 0
    i32.load offset=8
    local.get 2
    call 13
    unreachable)
  (func (;22;) (type 1) (param i32 i32 i32) (result i32)
    (local i32)
    local.get 2
    if  ;; label = @1
      local.get 0
      local.set 3
      loop  ;; label = @2
        local.get 3
        local.get 1
        i32.load8_u
        i32.store8
        local.get 3
        i32.const 1
        i32.add
        local.set 3
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 2
        i32.const -1
        i32.add
        local.tee 2
        br_if 0 (;@2;)
      end
    end
    local.get 0)
  (func (;23;) (type 2) (param i32 i32)
    (local i32)
    global.get 0
    i32.const 16
    i32.sub
    local.tee 2
    global.set 0
    local.get 2
    local.get 1
    i32.store offset=12
    local.get 2
    local.get 0
    i32.store offset=8
    local.get 2
    i32.const 1048992
    i32.store offset=4
    local.get 2
    i32.const 1048992
    i32.store
    local.get 2
    call 21
    unreachable)
  (func (;24;) (type 3) (param i32)
    (local i32)
    block  ;; label = @1
      local.get 0
      i32.load offset=4
      local.tee 1
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      i32.const 8
      i32.add
      i32.load
      i32.eqz
      br_if 0 (;@1;)
      local.get 1
      call 1
    end)
  (func (;25;) (type 3) (param i32)
    local.get 0
    i32.const 4
    i32.add
    i32.load
    if  ;; label = @1
      local.get 0
      i32.load
      call 1
    end)
  (func (;26;) (type 0) (param i32 i32) (result i32)
    block (result i32)  ;; label = @1
      i32.const 8
      local.get 1
      i32.lt_u
      if  ;; label = @2
        local.get 1
        local.get 0
        call 7
        br 1 (;@1;)
      end
      local.get 0
      call 0
    end)
  (func (;27;) (type 7)
    i32.const 1048959
    i32.const 17
    i32.const 1048976
    call 19
    unreachable)
  (func (;28;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i32.load
    drop
    loop  ;; label = @1
      br 0 (;@1;)
    end
    unreachable)
  (func (;29;) (type 0) (param i32 i32) (result i32)
    local.get 0
    i64.load32_u
    local.get 1
    call 9)
  (func (;30;) (type 2) (param i32 i32)
    local.get 0
    local.get 1
    i32.const 1049368
    i32.load
    local.tee 0
    i32.const 1
    local.get 0
    select
    call_indirect (type 2)
    unreachable)
  (func (;31;) (type 5) (param i32) (result i32)
    local.get 0
    i32.const 4
    i32.add
    i32.load)
  (func (;32;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
  (func (;33;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.sub)
  (func (;34;) (type 0) (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.mul)
  (func (;35;) (type 6) (param i32) (result i64)
    i64.const 2126898348991892564)
  (func (;36;) (type 6) (param i32) (result i64)
    i64.const 680220509154076087)
  (func (;37;) (type 3) (param i32)
    nop)
  (func (;38;) (type 2) (param i32 i32)
    nop)
  (table (;0;) 16 16 funcref)
  (memory (;0;) 17)
  (global (;0;) (mut i32) (i32.const 1048576))
  (export "memory" (memory 0))
  (export "div_i32" (func 20))
  (export "sub_i32" (func 33))
  (export "mult_i32" (func 34))
  (export "add_i32" (func 32))
  (elem (;0;) (i32.const 1) func 38 37 14 6 16 36 24 11 12 25 35 29 28 37 36)
  (data (;0;) (i32.const 1048576) "simple/src/lib.rs\00\00\00\00\00\10\00\11\00\00\00B\00\00\00\0c")
  (data (;1;) (i32.const 1048624) "attempt to divide by zero\00\00\00\00\00\00\00attempt to divide with overflow\00\02\00\00\00\04\00\00\00\04\00\00\00\03\00\00\00\04\00\00\00\05\00\00\00\02\00\00\00\00\00\00\00\01\00\00\00\06\00\00\00called `Option::unwrap()` on a `None` valueinternal error: entered unreachable codesrc/libstd/lib.rs\eb\00\10\00\11\00\00\00\01\00\00\00\01\00\00\00src/libstd/panicking.rs\00\0c\01\10\00\17\00\00\00\a1\01\00\00\0f\00\00\00\0c\01\10\00\17\00\00\00\a2\01\00\00\0f\00\00\00\07\00\00\00\10\00\00\00\04\00\00\00\08\00\00\00\09\00\00\00\0a\00\00\00\0c\00\00\00\04\00\00\00\0b\00\00\00src/liballoc/raw_vec.rscapacity overflowh\01\10\00\17\00\00\00\ee\02\00\00\05\00\00\00\0e\00\00\00\00\00\00\00\01\00\00\00\0f\00\00\00index out of bounds: the len is  but the index is \00\00\b0\01\10\00 \00\00\00\d0\01\10\00\12\00\00\0000010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899src/libcore/fmt/mod.rs\00\00\bc\02\10\00\16\00\00\00L\04\00\00\11\00\00\00\bc\02\10\00\16\00\00\00V\04\00\00$"))
