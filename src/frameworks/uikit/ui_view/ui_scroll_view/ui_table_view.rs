/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UITableView`.
//!
//! This currently models table geometry and scrolling, but not cell rendering.

use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect, CGSize};
use crate::frameworks::foundation::NSInteger;
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, nil, objc_classes, todo_objc_setter, ClassExports,
    NSZonePtr,
};

type UITableViewStyle = NSInteger;
const UITableViewStylePlain: UITableViewStyle = 0;
type UITableViewCellSeparatorStyle = NSInteger;
type UITableViewScrollPosition = NSInteger;
const UITableViewScrollPositionNone: UITableViewScrollPosition = 0;
const UITableViewScrollPositionTop: UITableViewScrollPosition = 1;
const UITableViewScrollPositionMiddle: UITableViewScrollPosition = 2;
const UITableViewScrollPositionBottom: UITableViewScrollPosition = 3;

pub struct UITableViewHostObject {
    superclass: super::UIScrollViewHostObject,
    /// UITableViewDataSource, weak reference
    data_source: id,
    style: UITableViewStyle,
    row_height: CGFloat,
    section_row_counts: Vec<NSInteger>,
}
impl_HostObject_with_superclass!(UITableViewHostObject);
impl Default for UITableViewHostObject {
    fn default() -> Self {
        UITableViewHostObject {
            superclass: Default::default(),
            data_source: nil,
            style: UITableViewStylePlain,
            row_height: 44.0,
            section_row_counts: Vec::new(),
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UITableView: UIScrollView

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UITableViewHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)dataSource {
    env.objc.borrow::<UITableViewHostObject>(this).data_source
}
- (())setDataSource:(id)data_source {
    env.objc.borrow_mut::<UITableViewHostObject>(this).data_source = data_source;
}

- (id)initWithFrame:(CGRect)frame style:(UITableViewStyle)style {
    let this: id = msg![env; this initWithFrame:frame];
    env.objc.borrow_mut::<UITableViewHostObject>(this).style = style;
    this
}
- (UITableViewStyle)style {
    env.objc.borrow::<UITableViewHostObject>(this).style
}

- (CGFloat)rowHeight {
    env.objc.borrow::<UITableViewHostObject>(this).row_height
}
- (())setRowHeight:(CGFloat)row_height {
    env.objc.borrow_mut::<UITableViewHostObject>(this).row_height = row_height;
}

// Nothing is drawn yet, so these do nothing.
- (())setSeparatorStyle:(UITableViewCellSeparatorStyle)style {
    todo_objc_setter!(this, style);
}
- (())setSeparatorColor:(id)color {
    todo_objc_setter!(this, color);
}
- (())setSectionHeaderHeight:(CGFloat)height {
    todo_objc_setter!(this, height);
}
- (())setSectionFooterHeight:(CGFloat)height {
    todo_objc_setter!(this, height);
}
- (())setAllowsSelection:(bool)allows {
    todo_objc_setter!(this, allows);
}
- (())setTableHeaderView:(id)view {
    todo_objc_setter!(this, view);
}
- (())setTableFooterView:(id)view {
    todo_objc_setter!(this, view);
}
- (())setBackgroundView:(id)view {
    todo_objc_setter!(this, view);
}
- (())reloadData {
    let data_source = env.objc.borrow::<UITableViewHostObject>(this).data_source;
    let mut section_row_counts = Vec::new();
    if data_source != nil {
        let sections_selector = env.objc.register_host_selector(
            "numberOfSectionsInTableView:".to_string(),
            &mut env.mem,
        );
        let provides_sections: bool = msg![env; data_source respondsToSelector:sections_selector];
        let section_count: NSInteger = if provides_sections {
            msg![env; data_source numberOfSectionsInTableView:this]
        } else {
            1
        };
        assert!(
            section_count >= 0,
            "UITableView data source returned a negative section count"
        );
        for section in 0..section_count {
            let row_count: NSInteger =
                msg![env; data_source tableView:this numberOfRowsInSection:section];
            assert!(
                row_count >= 0,
                "UITableView data source returned a negative row count"
            );
            section_row_counts.push(row_count);
        }
    }

    let row_height = env.objc.borrow::<UITableViewHostObject>(this).row_height;
    let total_rows: NSInteger = section_row_counts.iter().copied().sum();
    env.objc.borrow_mut::<UITableViewHostObject>(this).section_row_counts = section_row_counts;
    let bounds: CGRect = msg![env; this bounds];
    let size = CGSize {
        width: bounds.size.width,
        height: total_rows as CGFloat * row_height,
    };
    () = msg![env; this setContentSize:size];
    log_dbg!(
        "[(UITableView*) {:?} reloadData] laid out {} uniform rows",
        this,
        total_rows
    );
}

- (())scrollToRowAtIndexPath:(id)index_path
            atScrollPosition:(UITableViewScrollPosition)position
                    animated:(bool)_animated {
    assert_ne!(index_path, nil, "UITableView cannot scroll to a nil index path");
    let section: NSInteger = msg![env; index_path section];
    let row: NSInteger = msg![env; index_path row];
    let (row_height, content_height, row_start, current_offset) = {
        let host = env.objc.borrow::<UITableViewHostObject>(this);
        assert!(
            section >= 0 && (section as usize) < host.section_row_counts.len(),
            "UITableView section is out of range"
        );
        let row_count = host.section_row_counts[section as usize];
        assert!(
            row >= 0 && row < row_count,
            "UITableView row is out of range"
        );
        let preceding_rows: NSInteger = host.section_row_counts[..section as usize]
            .iter()
            .copied()
            .sum();
        let row_height = host.row_height;
        let content_height = host
            .section_row_counts
            .iter()
            .copied()
            .sum::<NSInteger>() as CGFloat
            * row_height;
        let row_start = (preceding_rows + row) as CGFloat * row_height;
        (
            row_height,
            content_height,
            row_start,
            host.superclass.content_offset,
        )
    };

    let bounds: CGRect = msg![env; this bounds];
    let target = match position {
        UITableViewScrollPositionNone => {
            if row_start < current_offset.y {
                row_start
            } else if row_start + row_height > current_offset.y + bounds.size.height {
                row_start + row_height - bounds.size.height
            } else {
                current_offset.y
            }
        }
        UITableViewScrollPositionTop => row_start,
        UITableViewScrollPositionMiddle => row_start - (bounds.size.height - row_height) / 2.0,
        UITableViewScrollPositionBottom => row_start - (bounds.size.height - row_height),
        _ => panic!("Unknown UITableViewScrollPosition {position}"),
    };
    let max_offset = (content_height - bounds.size.height).max(0.0);
    let target = target.max(0.0).min(max_offset);
    if target != current_offset.y {
        let offset = CGPoint {
            x: current_offset.x,
            y: target,
        };
        () = msg![env; this setContentOffset:offset];
    }
}
- (())deselectRowAtIndexPath:(id)_index_path animated:(bool)_animated {
    // No row can be selected.
}

@end

};
