//! N-dimensional convolution layers.
use super::Path;
use crate::Tensor;
use std::borrow::Borrow;

#[derive(Debug, Clone, Copy)]
pub struct ConvConfigND<ND> {
    pub stride: ND,
    pub padding: ND,
    pub dilation: ND,
    pub groups: i64,
    pub bias: bool,
    pub ws_init: super::Init,
    pub bs_init: super::Init,
}

pub type ConvConfig = ConvConfigND<i64>;

impl Default for ConvConfig {
    fn default() -> Self {
        ConvConfig {
            stride: 1,
            padding: 0,
            dilation: 1,
            groups: 1,
            bias: true,
            ws_init: super::Init::KaimingUniform,
            bs_init: super::Init::Const(0.),
        }
    }
}

impl Default for ConvConfigND<[i64; 2]> {
    fn default() -> Self {
        ConvConfigND::<[i64; 2]> {
            stride: [1, 1],
            padding: [0, 0],
            dilation: [1, 1],
            groups: 1,
            bias: true,
            ws_init: super::Init::KaimingUniform,
            bs_init: super::Init::Const(0.),
        }
    }
}

// Use const generics when they have landed in stable rust.
/// A N-dimensional convolution layer.
#[derive(Debug)]
pub struct Conv<ND> {
    pub ws: Tensor,
    pub bs: Tensor,
    config: ConvConfigND<ND>,
}

/// One dimension convolution layer.
pub type Conv1D = Conv<[i64; 1]>;

/// Two dimensions convolution layer.
pub type Conv2D = Conv<[i64; 2]>;

/// Three dimensions convolution layer.
pub type Conv3D = Conv<[i64; 3]>;

trait Create: std::convert::AsRef<[i64]> + std::marker::Sized {
    fn make_array(i: i64) -> Self;

    fn conv_nd<'a, T: Borrow<super::Path<'a>>>(
        vs: T,
        in_dim: i64,
        out_dim: i64,
        ksizes: Self,
        config: ConvConfigND<Self>,
    ) -> Conv<Self> {
        let vs = vs.borrow();
        let bs = if config.bias {
            vs.var("bias", &[out_dim], config.bs_init)
        } else {
            Tensor::zeros(&[out_dim], (crate::Kind::Float, vs.device()))
        };
        let mut weight_size = vec![out_dim, in_dim];
        weight_size.extend(ksizes.as_ref().iter());
        let ws = vs.var("weight", weight_size.as_slice(), config.ws_init);
        Conv { ws, bs, config }
    }

    fn conv<'a, T: Borrow<super::Path<'a>>>(
        vs: T,
        in_dim: i64,
        out_dim: i64,
        ksize: i64,
        config: ConvConfig,
    ) -> Conv<Self> {
        let config = ConvConfigND::<Self> {
            stride: Self::make_array(config.stride),
            padding: Self::make_array(config.padding),
            dilation: Self::make_array(config.dilation),
            groups: config.groups,
            bias: config.bias,
            ws_init: config.ws_init,
            bs_init: config.bs_init,
        };
        Self::conv_nd(vs, in_dim, out_dim, Self::make_array(ksize), config)
    }
}

impl Create for [i64; 1] {
    fn make_array(i: i64) -> Self {
        [i]
    }
}

impl Create for [i64; 2] {
    fn make_array(i: i64) -> Self {
        [i, i]
    }
}

impl Create for [i64; 3] {
    fn make_array(i: i64) -> Self {
        [i, i, i]
    }
}

impl Conv1D {
    pub fn new<'a, T: Borrow<Path<'a>>>(vs: T, i: i64, o: i64, k: i64, c: ConvConfig) -> Self {
        <[i64; 1]>::conv(vs, i, o, k, c)
    }
}

impl Conv2D {
    pub fn new<'a, T: Borrow<Path<'a>>>(vs: T, i: i64, o: i64, k: i64, c: ConvConfig) -> Self {
        <[i64; 2]>::conv(vs, i, o, k, c)
    }

    pub fn new_nd<'a, T: Borrow<Path<'a>>>(
        vs: T,
        i: i64,
        o: i64,
        k: [i64; 2],
        c: ConvConfigND<[i64; 2]>,
    ) -> Self {
        <[i64; 2]>::conv_nd(vs, i, o, k, c)
    }
}

impl Conv3D {
    pub fn new<'a, T: Borrow<Path<'a>>>(vs: T, i: i64, o: i64, k: i64, c: ConvConfig) -> Self {
        <[i64; 3]>::conv(vs, i, o, k, c)
    }

    pub fn new_nd<'a, T: Borrow<Path<'a>>>(
        vs: T,
        i: i64,
        o: i64,
        k: [i64; 3],
        c: ConvConfigND<[i64; 3]>,
    ) -> Self {
        <[i64; 3]>::conv_nd(vs, i, o, k, c)
    }
}

impl super::module::Module for Conv1D {
    fn forward(&self, xs: &Tensor) -> Tensor {
        Tensor::conv1d(
            &xs,
            &self.ws,
            &self.bs,
            &self.config.stride,
            &self.config.padding,
            &self.config.dilation,
            self.config.groups,
        )
    }
}

impl super::module::Module for Conv2D {
    fn forward(&self, xs: &Tensor) -> Tensor {
        Tensor::conv2d(
            &xs,
            &self.ws,
            &self.bs,
            &self.config.stride,
            &self.config.padding,
            &self.config.dilation,
            self.config.groups,
        )
    }
}

impl super::module::Module for Conv3D {
    fn forward(&self, xs: &Tensor) -> Tensor {
        Tensor::conv3d(
            &xs,
            &self.ws,
            &self.bs,
            &self.config.stride,
            &self.config.padding,
            &self.config.dilation,
            self.config.groups,
        )
    }
}