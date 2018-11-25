use neon::prelude::*;

// trace_macros!(true);

pub trait ToJs {
    fn to_js<'a, CX>(&self, cx: &mut CX) -> Handle<'a, JsValue>
    where
        Self: 'a,
        CX: Context<'a>;
}

impl ToJs for f64 {
    fn to_js<'a, CX>(&self, cx: &mut CX) -> Handle<'a, JsValue>
    where
        Self: 'a,
        CX: Context<'a>,
    {
        neon_serde::to_value(cx, self).unwrap()
    }
}

impl ToJs for str {
    fn to_js<'a, CX>(&self, cx: &mut CX) -> Handle<'a, JsValue>
    where
        Self: 'a,
        CX: Context<'a>,
    {
        neon_serde::to_value(cx, self).unwrap()
    }
}

impl<'b> ToJs for Handle<'b, JsValue> {
    fn to_js<'a, CX>(&self, cx: &mut CX) -> Handle<'a, JsValue>
    where
        Self: 'a,
        CX: Context<'a>,
    {
        self.clone()
    }
}

macro_rules! js {
    ( @chain, $cx:expr, $value:expr , ) => {{
        $value
    }};
    ( @chain, $cx:expr, $value:expr , . $key:ident = $( $rest:tt )* ) => {{
        let value = $value;
        let rest = js!($cx, $( $rest )*);
        value
            .downcast::<JsObject>()
            .unwrap()
            .set($cx, stringify!($key), rest)
            .unwrap()
    }};
    ( @chain, $cx:expr, $value:expr , . $key:ident ( $( $args:expr ),* ) $( $rest:tt )* ) => {{
        js!(@chain,
             $cx,
             {
                 let value = $value;
                 let function = value
                     .downcast::<JsObject>()
                     .unwrap()
                     .get($cx, stringify!($key))
                     .unwrap();
                 let mut args = vec![];
                 {
                     $( args.push(js!($cx, $args)); )*
                 }
                 function
                     .downcast::<JsFunction>()
                     .unwrap()
                     .call($cx, value, args)
                     .unwrap()
             },
             $( $rest )*)
    }};
    ( @chain, $cx:expr, $value:expr , . $key:ident $( $rest:tt )* ) => {{
        let value = $value;
        js!(@chain,
             $cx,
             value
             .downcast::<JsObject>()
             .unwrap()
             .get($cx, stringify!($key))
             .unwrap(),
             $( $rest )*)
    }};
    ( @chain, $cx:expr, $value:expr , ( $( $args:expr ),* ) $( $rest:tt )* ) => {{
        js!(@chain,
             $cx,
             {
                 let value = $value;
                 let mut args = vec![];
                 {
                     $( args.push(js!($cx, $args)); )*
                 }
                 let null = ($cx).null();
                 value
                     .downcast::<JsFunction>()
                     .unwrap()
                     .call($cx, null, args)
                     .unwrap()
             },
             $( $rest )*)
    }};
    ( $cx:expr, $value:ident $( $rest:tt )+ ) => {{
        js!(@chain,
             $cx,
             $value,
             $( $rest )*)
    }};
    ( $cx:expr, null ) => {{
        ($cx).null()
    }};
    ( $cx:expr, $expr:expr ) => {{
        ($expr).to_js($cx)
    }}
}

macro_rules! js_ {
    ( $cx:expr, $( $args:tt )* ) => {{
        let value = js!( $cx, $( $args )* );
        neon_serde::from_value($cx, value).unwrap()
    }};
}
