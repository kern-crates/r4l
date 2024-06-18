# R4L: Rust OS Compatible with Linux

## History
In the earliest design, we proposed OSL (Operating System Layer). The interfaces and organization
of OSL were not based on any single OS as a reference standard; rather, OSL defined its own more
general interfaces. During driver development, interaction with the OS had to follow OSL interface
specifications. This way, as long as the target OS was compatible with OSL, the driver would naturally
be compatible as well. This is what we called the cross-kernel driver framework and implementation.

The benefit of this approach is that hardware manufacturers could focus solely on the driver itself
without needing to learn various different OSs and then adapt to each one. 
The project maintenance address is: https://github.com/kern-crates/.github/pull/7/files.

However, this approach has a significant drawback: it requires acceptance from all hardware manufacturers.
In reality, hardware manufacturers are more inclined to adapt to mainstream OSs like Linux/Windows, and the
hardware ecosystem of these mainstream OSs is already relatively stable. Therefore, we proposed a second solution.
Based on the idea of the first solution, we let OSL follow the framework of a particular OS (Linux).
This is the basis of this project.
