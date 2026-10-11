It might be easier to make the SlamKeyFrame require Globaltransform and Camera3d, then it can store it's own uncertainty from that.

The position uncertainty will be in its local coordinate frame.
Same as the rotation uncertainty.

Because we are likely to need to have a best guess for the rotation, this makes it more uniform between rotation and position. If we need to have a guess for rotation, then might as well guess position as well and store both of those in a transform. Def easier for visualisations as well