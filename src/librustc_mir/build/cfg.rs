// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.




//! Routines for manipulating the control-flow graph.

use build::CFG;
use rustc::mir::repr::*;
use syntax::codemap::Span;

impl<'tcx> CFG<'tcx> {
    pub fn block_data(&self, blk: BasicBlock) -> &BasicBlockData<'tcx> {
        &self.basic_blocks[blk.index()]
    }

    pub fn block_data_mut(&mut self, blk: BasicBlock) -> &mut BasicBlockData<'tcx> {
        &mut self.basic_blocks[blk.index()]
    }

    pub fn start_new_block(&mut self) -> BasicBlock {
        let node_index = self.basic_blocks.len();
        self.basic_blocks.push(BasicBlockData::new(None));
        BasicBlock::new(node_index)
    }

    pub fn start_new_cleanup_block(&mut self) -> BasicBlock {
        let bb = self.start_new_block();
        self.block_data_mut(bb).is_cleanup = true;
        bb
    }

    pub fn push(&mut self, block: BasicBlock, statement: Statement<'tcx>) {
        debug!("push({:?}, {:?})", block, statement);
        self.block_data_mut(block).statements.push(statement);
    }

    pub fn push_assign(&mut self,
                       block: BasicBlock,
                       span: Span,
                       lvalue: &Lvalue<'tcx>,
                       rvalue: Rvalue<'tcx>) {
        self.push(block, Statement {
            span: span,
            kind: StatementKind::Assign(lvalue.clone(), rvalue)
        });
    }

    pub fn push_assign_constant(&mut self,
                                block: BasicBlock,
                                span: Span,
                                temp: &Lvalue<'tcx>,
                                constant: Constant<'tcx>) {
        self.push_assign(block, span, temp, Rvalue::Use(Operand::Constant(constant)));
    }

    pub fn push_assign_unit(&mut self,
                            block: BasicBlock,
                            span: Span,
                            lvalue: &Lvalue<'tcx>) {
        self.push_assign(block, span, lvalue, Rvalue::Aggregate(
            AggregateKind::Tuple, vec![]
        ));
    }

    pub fn terminate(&mut self,
                     block: BasicBlock,
                     terminator: Terminator<'tcx>) {
        debug_assert!(self.block_data(block).terminator.is_none(),
                      "terminate: block {:?} already has a terminator set", block);
        self.block_data_mut(block).terminator = Some(terminator);
    }
}
