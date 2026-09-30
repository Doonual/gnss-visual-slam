Gaussian distributions AKA multivariate normal distributions are used to estimate pose.

## Parameters
Gaussian distributions are parameterised by 2 matrices
- $\mu$ - Mean, Column vector equal to number of dimensions
- $\Sigma$ - Covariance, Square vector equal to number of dimensions

## Definition
Their probability density function is defined as
$$
\huge
2\pi^{-\frac{k}{2}} \times \text{det}(\Sigma)^{-\frac12} \exp{\left(-\frac12\left(x-\mu\right)^T\Sigma^{-1}(x-\mu)\right)}
$$

## Mean
Mean determines the centre of the Gaussian distribution.
## Covariance
