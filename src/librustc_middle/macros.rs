#[macro_export]
macro_rules! bug {
    () => ( bug!("impossible case reached") );
    ($($message:tt)*) => ({
        $crate::util::bug::bug_fmt(file!(), line!(), format_args!($($message)*))
    })
}

#[macro_export]
macro_rules! span_bug {
    ($span:expr, $($message:tt)*) => ({
        $crate::util::bug::span_bug_fmt(file!(), line!(), $span, format_args!($($message)*))
    })
}

///////////////////////////////////////////////////////////////////////////
// Lift and TypeFoldable macros
//
// When possible, use one of these (relatively) convenient macros to write
// the impls for you.

#[macro_export]
macro_rules! CloneLiftImpls {
    (for <$tcx:lifetime> { $($ty:ty,)+ }) => {
        $(
            impl<$tcx> $crate::ty::Lift<$tcx> for $ty {
                type Lifted = Self;
                fn lift_to_tcx(&self, _: $crate::ty::TyCtxt<$tcx>) -> Option<Self> {
                    Some(Clone::clone(self))
                }
            }
        )+
    };

    ($($ty:ty,)+) => {
        CloneLiftImpls! {
            for <'tcx> {
                $($ty,)+
            }
        }
    };
}

/// Used for types that are `Copy` and which **do not care arena
/// allocated data** (i.e., don't need to be folded).
#[macro_export]
macro_rules! CloneTypeFoldableImpls {
    (for <$tcx:lifetime> { $($ty:ty,)+ }) => {
        $(
            impl<$tcx> $crate::ty::fold::TypeFoldable<$tcx> for $ty {
                fn super_fold_with<F: $crate::ty::fold::TypeFolder<$tcx>>(
                    &self,
                    _: &mut F
                ) -> $ty {
                    Clone::clone(self)
                }

                fn super_visit_with<F: $crate::ty::fold::TypeVisitor<$tcx>>(
                    &self,
                    _: &mut F)
                    -> bool
                {
                    false
                }
            }
        )+
    };

    ($($ty:ty,)+) => {
        CloneTypeFoldableImpls! {
            for <'tcx> {
                $($ty,)+
            }
        }
    };
}

#[macro_export]
macro_rules! CloneTypeFoldableAndLiftImpls {
    ($($t:tt)*) => {
        CloneTypeFoldableImpls! { $($t)* }
        CloneLiftImpls! { $($t)* }
    }
}

#[macro_export]
macro_rules! EnumTypeFoldableImpl {
    (impl<$($p:tt),*> TypeFoldable<$tcx:tt> for $s:path {
        $($variants:tt)*
    } $(where $($wc:tt)*)*) => {
        impl<$($p),*> $crate::ty::fold::TypeFoldable<$tcx> for $s
            $(where $($wc)*)*
        {
            fn super_fold_with<V: $crate::ty::fold::TypeFolder<$tcx>>(
                &self,
                folder: &mut V,
            ) -> Self {
                EnumTypeFoldableImpl!(@FoldVariants(self, folder) input($($variants)*) output())
            }

            fn super_visit_with<V: $crate::ty::fold::TypeVisitor<$tcx>>(
                &self,
                visitor: &mut V,
            ) -> bool {
                EnumTypeFoldableImpl!(@VisitVariants(self, visitor) input($($variants)*) output())
            }
        }
    };

    (@FoldVariants($this:expr, $folder:expr) input() output($($output:tt)*)) => {
        match $this {
            $($output)*
        }
    };

    (@FoldVariants($this:expr, $folder:expr)
     input( ($variant:path) ( $($variant_arg:ident),* ) , $($input:tt)*)
     output( $($output:tt)*) ) => {
        EnumTypeFoldableImpl!(
            @FoldVariants($this, $folder)
                input($($input)*)
                output(
                    $variant ( $($variant_arg),* ) => {
                        $variant (
                            $($crate::ty::fold::TypeFoldable::fold_with($variant_arg, $folder)),*
                        )
                    }
                    $($output)*
                )
        )
    };

    (@FoldVariants($this:expr, $folder:expr)
     input( ($variant:path) { $($variant_arg:ident),* $(,)? } , $($input:tt)*)
     output( $($output:tt)*) ) => {
        EnumTypeFoldableImpl!(
            @FoldVariants($this, $folder)
                input($($input)*)
                output(
                    $variant { $($variant_arg),* } => {
                        $variant {
                            $($variant_arg: $crate::ty::fold::TypeFoldable::fold_with(
                                $variant_arg, $folder
                            )),* }
                    }
                    $($output)*
                )
        )
    };

    (@FoldVariants($this:expr, $folder:expr)
     input( ($variant:path), $($input:tt)*)
     output( $($output:tt)*) ) => {
        EnumTypeFoldableImpl!(
            @FoldVariants($this, $folder)
                input($($input)*)
                output(
                    $variant => { $variant }
                    $($output)*
                )
        )
    };

    (@VisitVariants($this:expr, $visitor:expr) input() output($($output:tt)*)) => {
        match $this {
            $($output)*
        }
    };

    (@VisitVariants($this:expr, $visitor:expr)
     input( ($variant:path) ( $($variant_arg:ident),* ) , $($input:tt)*)
     output( $($output:tt)*) ) => {
        EnumTypeFoldableImpl!(
            @VisitVariants($this, $visitor)
                input($($input)*)
                output(
                    $variant ( $($variant_arg),* ) => {
                        false $(|| $crate::ty::fold::TypeFoldable::visit_with(
                            $variant_arg, $visitor
                        ))*
                    }
                    $($output)*
                )
        )
    };

    (@VisitVariants($this:expr, $visitor:expr)
     input( ($variant:path) { $($variant_arg:ident),* $(,)? } , $($input:tt)*)
     output( $($output:tt)*) ) => {
        EnumTypeFoldableImpl!(
            @VisitVariants($this, $visitor)
                input($($input)*)
                output(
                    $variant { $($variant_arg),* } => {
                        false $(|| $crate::ty::fold::TypeFoldable::visit_with(
                            $variant_arg, $visitor
                        ))*
                    }
                    $($output)*
                )
        )
    };

    (@VisitVariants($this:expr, $visitor:expr)
     input( ($variant:path), $($input:tt)*)
     output( $($output:tt)*) ) => {
        EnumTypeFoldableImpl!(
            @VisitVariants($this, $visitor)
                input($($input)*)
                output(
                    $variant => { false }
                    $($output)*
                )
        )
    };
}
