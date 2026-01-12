# Device Discovery

Cohete supports multiple methods for discovering and connecting to Jetson devices.

## Connection Methods

### USB-C Direct Connection

The simplest connection method. Connect your Jetson via USB-C and it appears at `192.168.55.1`:

```rust
use cohete::device::JetsonDevice;

let device = JetsonDevice::discover_usb().await?;
println!("Connected via USB: {}", device.id());
```

### Ethernet Connection

For networked Jetsons with known IP addresses:

```rust
use std::net::Ipv4Addr;

let ip = "192.168.1.100".parse().unwrap();
let device = JetsonDevice::connect(ip).await?;
```

### mDNS Discovery

Automatically discover Jetsons advertising via mDNS:

```rust
let devices = JetsonDevice::discover_mdns().await?;
for device in devices {
    println!("Found: {} at {:?}", device.id(), device.info().connection);
}
```

## Discover All

The `discover_all()` method combines all discovery methods:

```rust
let devices = JetsonDevice::discover_all().await?;
// Returns devices found via USB + mDNS
```

## Device Information

Once connected, you can query device details:

```rust
let info = device.info();

println!("ID: {}", info.id);
println!("Model: {}", info.model);
println!("JetPack: {:?}", info.jetpack_version);
println!("Hostname: {:?}", info.hostname);

// Model-specific information
println!("Memory: {} MB", info.model.memory_mb());
println!("CUDA Cores: {}", info.model.cuda_cores());
println!("AI Performance: {} TOPS", info.model.tops());
```

## Compute Hints

Get hints for trueno backend selection:

```rust
let hint = device.compute_hint();

println!("Prefer NEON: {}", hint.prefer_neon);
println!("Memory Budget: {} MB", hint.memory_budget_mb);
println!("CUDA Available: {}", hint.cuda_available);
```

## YAML Configuration

Configure discovery in your `cohete.yaml`:

```yaml
discovery:
  methods:
    - usb
    - mdns
    - static:
        - "192.168.1.101"
        - "192.168.1.102"
```

## Error Handling

```rust
match JetsonDevice::discover_usb().await {
    Ok(device) => println!("Found: {}", device.id()),
    Err(cohete::Error::DeviceNotFound(msg)) => {
        eprintln!("No USB device: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```
