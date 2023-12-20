
# 2D Transformation

- Scale

```
  | sx  0 | | x |
  | 0  sy | | y |
```

- Reflection

sx = 1 or -1, and sy = 1 or -1 to scale matrix.

- Shear

```
  | 1  a | | x |
  | 0  1 | | y |
```

- Rotate (about the origin(0, 0), CCW(逆时针) by default)

```
  | cos(t)  -sin(t) | | x |
  | sin(t)   cos(t) | | y |
```

Linear transformation with matrices:

```
| x' |   | a b | | x |
| y' | = | c d | | y |

    x' = M x
```

## Homogeneous coordinates(齐次坐标)

Add a third coordinate (w-coordinate)

- 2D point  = (x, y, 1)
- 2D vector = (x, y, 0)

vector + vector = vector
point - point   = vector
point + vector  = point
point + point   = middle point

```
| x |                 | x/w |
| y | is the 2D point | y/w |, w != 0
| w |                 |  1  |
```

Given affine map(仿射变换) = linear map + translation.

Using homogeneous coordinates represent all affine map:

```
| x' |   | a  b  tx | | x |
| y' | = | c  d  ty | | y |
| 1  |   | 0  0   1 | | 1 |
```

## composite transformations

The order of transformations is important, because matrix multiplication is not commutative(交换律).
And we can obtain a final matrix from multiplication of many transformations matrices.

For instance, point `P` will perform transformations of `A1, A2, ... An` in order:

```
An * ... * A1 * P = M(A) * P
```


# 3D Transformation

- 3D point  = (x, y, z, 1)
- 3D vector = (x, y, z, 0)

Using 4x4 matrices for affine transformations:

```
| x' |   | a  b  c  tx | | x |
| y' | = | d  e  f  ty | | y |
| z' |   | g  h  i  tz | | 1 |
| 1  |   | 0  0  c   1 | | 1 |
```

## 3D rotation

```
R_xyz(a, b, c) = Rx(a) Ry(b) Rz(c)
```

`a, b, c` are so-called euler angles.
Used in flight with `roll, picth, yaw`.

Rotation by angle around axis with `rodrigues's rotation formula`.

TODO: quaternion(四元数)

## View transformation

Define the camare:

- Postion `e`
- Look-at direction `g`
- Up direction `t`

In general, camera will always be at origin, up at Y, and look at -z.
And transform all the objects with the camera.

## Projection(投影)

### Orthographic(正交)

In general, mapping a cuboid(长方体) [left, right] x [bottom x top] x [far x near] to the canonical cube [-1, 1]x3.

Left and right are locate at axis x. Bottom and top are locate at axis y. Far and near are locate at axis z. Camera is look at from z to -z.

```
        ^ y
        |
        | / -z
        |/
  ------+-----> x
       /|
      / |
     z

we look at direction of -z, and n can be larger than 0.
Here n and f is coordinates at z axis and n is large than far(n > f).
```

- making translation and scale(平移和缩放) then to get orthographic projection matrix

```
          | 2/(r-l)  0        0        0 | | 1 0 0 -(r+l)/2 |   | 2/(r-l)  0        0        -(r+l)/(r-l) |
M_ortho = | 0        2/(t-b)  0        0 | | 0 1 0 -(t+b)/2 | = | 0        2/(t-b)  0        -(t+b)/(t-b) |
          | 0        0        2/(n-f)  0 | | 0 0 1 -(n+f)/2 |   | 0        0        2/(n-f)  -(n+f)/(n-f) |
          | 0        0        0        1 | | 0 0 0     1    |   | 0        0        0              1      |
```

### Perspective(透视)

> (x, y, z, 1) -> (xz, yz, z^2, z!=0), represent the same point(x, y, z) in 3D.

Squish(抗压) frustum(锥体) to cuboid with matrix of transformation, then making orthographic projection.

```
x' = n/z * x
y' = n/z * y

                  | n  0  0       0 |   | 2n/(r-l)  0         -(r+l)/(r-l)  0         |
M_persp = M_ortho*| 0  n  0       0 | = | 0         2n/(t-b)  -(t+b)/(r-b)  0         |
                  | 0  0  (n+f)  -nf|   | 0         0         (n+f)/(n-f)   -2nf/(n-f)|
                  | 0  0  1       0 |   | 0         0                   1   0         |
```

If the frustum is sysmmetric(r = -l, t = -b), then we have a simplified matrix.

```
          | n/r  0    0             0        |
M_persp = | 0    n/t  0             0        |
          | 0    0    (n+f)/(n-f)  -2nf(n-f) |
          | 0    0    1             0        |

```

- field of view(fovY): the angle of vertical view in Y axis(`tan(fov/2) = t / n`)
- aspect ratio: the ratio of width to height(`r / t`)

*More help about transformation please refer to [OpenGL Projection Matrix](http://www.songho.ca/opengl/gl_projectionmatrix_mathml.html).*
