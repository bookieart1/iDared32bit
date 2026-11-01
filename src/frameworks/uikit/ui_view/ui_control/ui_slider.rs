/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UISlider`.

use super::{send_actions, UIControlEventValueChanged, UIControlState, UIControlStateNormal};
use crate::frameworks::core_graphics::{CGPoint, CGRect, CGSize};
use crate::objc::{
    id, impl_HostObject_with_superclass, msg, msg_class, msg_super, nil, objc_classes, release,
    retain, ClassExports, NSZonePtr,
};
use crate::Environment;
use std::collections::HashMap;

pub struct UISliderHostObject {
    superclass: super::UIControlHostObject,
    value: f32,
    minimum_value: f32,
    maximum_value: f32,
    continuous: bool,
    minimum_value_image: id,
    maximum_value_image: id,
    minimum_track_view: id,
    maximum_track_view: id,
    thumb_view: id,
    thumb_images: HashMap<UIControlState, id>,
    minimum_track_images: HashMap<UIControlState, id>,
    maximum_track_images: HashMap<UIControlState, id>,
}
impl_HostObject_with_superclass!(UISliderHostObject);
impl Default for UISliderHostObject {
    fn default() -> Self {
        UISliderHostObject {
            superclass: Default::default(),
            value: 0.0,
            minimum_value: 0.0,
            maximum_value: 1.0,
            continuous: true,
            minimum_value_image: nil,
            maximum_value_image: nil,
            minimum_track_view: nil,
            maximum_track_view: nil,
            thumb_view: nil,
            thumb_images: HashMap::new(),
            minimum_track_images: HashMap::new(),
            maximum_track_images: HashMap::new(),
        }
    }
}

fn image_for_state(images: &HashMap<UIControlState, id>, state: UIControlState) -> id {
    images
        .get(&state)
        .or_else(|| images.get(&UIControlStateNormal))
        .copied()
        .unwrap_or(nil)
}

fn init_common(env: &mut Environment, this: id) -> id {
    let minimum_track_view: id = msg_class![env; UIImageView new];
    let maximum_track_view: id = msg_class![env; UIImageView new];
    let thumb_view: id = msg_class![env; UIImageView new];
    () = msg![env; this addSubview:maximum_track_view];
    () = msg![env; this addSubview:minimum_track_view];
    () = msg![env; this addSubview:thumb_view];
    let host = env.objc.borrow_mut::<UISliderHostObject>(this);
    host.minimum_track_view = minimum_track_view;
    host.maximum_track_view = maximum_track_view;
    host.thumb_view = thumb_view;
    () = msg![env; this setNeedsLayout];
    this
}

fn update_value_from_touch(env: &mut Environment, this: id, touch: id, event: id) {
    let point: CGPoint = msg![env; touch locationInView:this];
    let bounds: CGRect = msg![env; this bounds];
    let (minimum, maximum, continuous) = {
        let host = env.objc.borrow::<UISliderHostObject>(this);
        (host.minimum_value, host.maximum_value, host.continuous)
    };
    let fraction = if bounds.size.width > 0.0 {
        ((point.x - bounds.origin.x) / bounds.size.width).clamp(0.0, 1.0)
    } else {
        0.0
    };
    () = msg![env; this setValue:(minimum + fraction * (maximum - minimum))];
    if continuous {
        send_actions(env, this, event, UIControlEventValueChanged);
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UISlider: UIControl

+ (id)allocWithZone:(NSZonePtr)_zone {
    env.objc.alloc_object(this, Box::<UISliderHostObject>::default(), &mut env.mem)
}

- (id)initWithFrame:(CGRect)frame {
    let this: id = msg_super![env; this initWithFrame:frame];
    init_common(env, this)
}

- (id)initWithCoder:(id)coder {
    let this: id = msg_super![env; this initWithCoder:coder];
    init_common(env, this)
}

- (())dealloc {
    let UISliderHostObject {
        superclass: _, value: _, minimum_value: _, maximum_value: _, continuous: _,
        minimum_value_image, maximum_value_image, minimum_track_view, maximum_track_view,
        thumb_view, thumb_images, minimum_track_images, maximum_track_images,
    } = std::mem::take(env.objc.borrow_mut(this));
    release(env, minimum_value_image);
    release(env, maximum_value_image);
    release(env, minimum_track_view);
    release(env, maximum_track_view);
    release(env, thumb_view);
    for image in thumb_images.into_values() { release(env, image); }
    for image in minimum_track_images.into_values() { release(env, image); }
    for image in maximum_track_images.into_values() { release(env, image); }
    msg_super![env; this dealloc]
}

- (id)minimumValueImage { env.objc.borrow::<UISliderHostObject>(this).minimum_value_image }
- (())setMinimumValueImage:(id)image {
    retain(env, image);
    let old = std::mem::replace(&mut env.objc.borrow_mut::<UISliderHostObject>(this).minimum_value_image, image);
    release(env, old);
}
- (id)maximumValueImage { env.objc.borrow::<UISliderHostObject>(this).maximum_value_image }
- (())setMaximumValueImage:(id)image {
    retain(env, image);
    let old = std::mem::replace(&mut env.objc.borrow_mut::<UISliderHostObject>(this).maximum_value_image, image);
    release(env, old);
}

- (f32)minimumValue { env.objc.borrow::<UISliderHostObject>(this).minimum_value }
- (())setMinimumValue:(f32)value {
    let current = {
        let host = env.objc.borrow_mut::<UISliderHostObject>(this);
        host.minimum_value = value;
        host.value
    };
    () = msg![env; this setValue:current];
}
- (f32)maximumValue { env.objc.borrow::<UISliderHostObject>(this).maximum_value }
- (())setMaximumValue:(f32)value {
    let current = {
        let host = env.objc.borrow_mut::<UISliderHostObject>(this);
        host.maximum_value = value;
        host.value
    };
    () = msg![env; this setValue:current];
}
- (f32)value { env.objc.borrow::<UISliderHostObject>(this).value }
- (())setValue:(f32)value {
    let host = env.objc.borrow_mut::<UISliderHostObject>(this);
    host.value = value.min(host.maximum_value).max(host.minimum_value);
    () = msg![env; this setNeedsLayout];
}
- (())setValue:(f32)value animated:(bool)_animated { () = msg![env; this setValue:value]; }
- (bool)isContinuous { env.objc.borrow::<UISliderHostObject>(this).continuous }
- (())setContinuous:(bool)continuous {
    env.objc.borrow_mut::<UISliderHostObject>(this).continuous = continuous;
}

- (id)thumbImageForState:(UIControlState)state {
    image_for_state(&env.objc.borrow::<UISliderHostObject>(this).thumb_images, state)
}
- (())setThumbImage:(id)image forState:(UIControlState)state {
    retain(env, image);
    let old = env.objc.borrow_mut::<UISliderHostObject>(this).thumb_images.insert(state, image);
    if let Some(old) = old { release(env, old); }
    () = msg![env; this setNeedsLayout];
}
- (id)minimumTrackImageForState:(UIControlState)state {
    image_for_state(&env.objc.borrow::<UISliderHostObject>(this).minimum_track_images, state)
}
- (())setMinimumTrackImage:(id)image forState:(UIControlState)state {
    retain(env, image);
    let old = env.objc.borrow_mut::<UISliderHostObject>(this).minimum_track_images.insert(state, image);
    if let Some(old) = old { release(env, old); }
    () = msg![env; this setNeedsLayout];
}
- (id)maximumTrackImageForState:(UIControlState)state {
    image_for_state(&env.objc.borrow::<UISliderHostObject>(this).maximum_track_images, state)
}
- (())setMaximumTrackImage:(id)image forState:(UIControlState)state {
    retain(env, image);
    let old = env.objc.borrow_mut::<UISliderHostObject>(this).maximum_track_images.insert(state, image);
    if let Some(old) = old { release(env, old); }
    () = msg![env; this setNeedsLayout];
}

- (())layoutSubviews {
    let bounds: CGRect = msg![env; this bounds];
    let state: UIControlState = msg![env; this state];
    let (minimum, maximum, value, minimum_view, maximum_view, thumb_view, minimum_image, maximum_image, thumb_image) = {
        let host = env.objc.borrow::<UISliderHostObject>(this);
        (host.minimum_value, host.maximum_value, host.value, host.minimum_track_view,
         host.maximum_track_view, host.thumb_view, image_for_state(&host.minimum_track_images, state),
         image_for_state(&host.maximum_track_images, state), image_for_state(&host.thumb_images, state))
    };
    let fraction = if maximum > minimum { ((value - minimum) / (maximum - minimum)).clamp(0.0, 1.0) } else { 0.0 };
    let thumb_size: CGSize = if thumb_image == nil { CGSize { width: 20.0, height: 20.0 } } else { msg![env; thumb_image size] };
    let track_height = 4.0f32.min(bounds.size.height.max(0.0));
    let center_y = bounds.origin.y + bounds.size.height / 2.0;
    let thumb_center_x = bounds.origin.x + fraction * bounds.size.width;
    let minimum_width = (thumb_center_x - bounds.origin.x).max(0.0);
    let maximum_width = (bounds.size.width - minimum_width).max(0.0);
    () = msg![env; minimum_view setImage:minimum_image];
    () = msg![env; maximum_view setImage:maximum_image];
    () = msg![env; thumb_view setImage:thumb_image];
    () = msg![env; minimum_view setFrame:(CGRect { origin: CGPoint { x: bounds.origin.x, y: center_y - track_height / 2.0 }, size: CGSize { width: minimum_width, height: track_height } })];
    () = msg![env; maximum_view setFrame:(CGRect { origin: CGPoint { x: thumb_center_x, y: center_y - track_height / 2.0 }, size: CGSize { width: maximum_width, height: track_height } })];
    () = msg![env; thumb_view setFrame:(CGRect { origin: CGPoint { x: thumb_center_x - thumb_size.width / 2.0, y: center_y - thumb_size.height / 2.0 }, size: thumb_size })];
    let clear: id = msg_class![env; UIColor clearColor];
    let blue: id = msg_class![env; UIColor blueColor];
    let gray: id = msg_class![env; UIColor lightGrayColor];
    let white: id = msg_class![env; UIColor whiteColor];
    () = msg![env; minimum_view setBackgroundColor:(if minimum_image == nil { blue } else { clear })];
    () = msg![env; maximum_view setBackgroundColor:(if maximum_image == nil { gray } else { clear })];
    () = msg![env; thumb_view setBackgroundColor:(if thumb_image == nil { white } else { clear })];
}

- (id)hitTest:(CGPoint)point withEvent:(id)event {
    if msg![env; this pointInside:point withEvent:event] { this } else { nil }
}
- (bool)beginTrackingWithTouch:(id)touch withEvent:(id)event {
    update_value_from_touch(env, this, touch, event);
    true
}
- (bool)continueTrackingWithTouch:(id)touch withEvent:(id)event {
    update_value_from_touch(env, this, touch, event);
    true
}
- (())endTrackingWithTouch:(id)touch withEvent:(id)event {
    update_value_from_touch(env, this, touch, event);
    if !env.objc.borrow::<UISliderHostObject>(this).continuous {
        send_actions(env, this, event, UIControlEventValueChanged);
    }
    () = msg_super![env; this endTrackingWithTouch:touch withEvent:event];
}

@end

};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objc::release;
    use crate::{msg_class, Environment};

    #[test]
    fn slider_state_and_standard_subviews_are_functional() {
        let mut env = Environment::new_fake_for_test_with_thread();
        let env = &mut env;
        let stack = env.mem.alloc(0x1000);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = stack.to_bits() + 0x1000;
        let slider: id = msg_class![env; UISlider alloc];
        let slider: id = msg![env; slider initWithFrame:(CGRect { origin: CGPoint { x: 0.0, y: 0.0 }, size: CGSize { width: 100.0, height: 30.0 } })];
        assert!(msg![env; slider isContinuous]);
        () = msg![env; slider setContinuous:false];
        () = msg![env; slider setMinimumValue:10.0f32];
        () = msg![env; slider setMaximumValue:20.0f32];
        () = msg![env; slider setValue:15.0f32];
        let value: f32 = msg![env; slider value];
        assert_eq!(value, 15.0f32);
        () = msg![env; slider layoutSubviews];
        release(env, slider);
        env.cpu.regs_mut()[crate::cpu::Cpu::SP] = 0;
        env.mem.free(stack);
    }
}
