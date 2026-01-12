# Troubleshooting

Common issues and solutions when using cohete.

## Device Discovery

### USB device not found

**Symptoms:**
```
Error: DeviceNotFound("No USB device at 192.168.55.1")
```

**Solutions:**
1. Ensure USB-C cable is connected to the Jetson's USB-C port (not the barrel jack)
2. Check that the Jetson is booted and showing the login screen
3. Verify USB gadget mode is enabled:
   ```bash
   # On Jetson
   ifconfig usb0
   # Should show 192.168.55.1
   ```
4. Try a different USB-C cable (data cables required, not charge-only)

### mDNS discovery returns no devices

**Solutions:**
1. Ensure Avahi is running on the Jetson:
   ```bash
   sudo systemctl status avahi-daemon
   ```
2. Check firewall allows mDNS (port 5353 UDP)
3. Verify devices are on the same network segment

## Thermal Issues

### Immediate throttling on start

**Symptoms:**
- Circuit breaker opens immediately
- "Thermal threshold exceeded" errors

**Solutions:**
1. Ensure heatsink is properly mounted
2. Check thermal paste application
3. Use `ThermalPolicy::aggressive()` if cooling is adequate
4. Add active cooling (fan) for sustained workloads

### Frequent thermal throttling during inference

**Solutions:**
1. Reduce batch size
2. Add delays between inference batches
3. Lower context length to reduce memory bandwidth
4. Consider a more aggressive quantization level

## Memory Issues

### InsufficientMemory error

**Symptoms:**
```
Error: InsufficientMemory { requested_mb: 6000, available_mb: 4000 }
```

**Solutions:**
1. Use a more aggressive quantization level:
   ```rust
   let level = JetsonQuantizer::select_for_budget(model_size, &budget);
   ```
2. Reduce context length (smaller KV cache)
3. Ensure no other processes are using GPU memory:
   ```bash
   tegrastats  # Check memory usage
   ```

### Out of memory during inference

**Solutions:**
1. Set lower `memory_budget_mb` in configuration
2. Kill other GPU processes
3. Reduce `max_batch_size` in inference config
4. Add swap on NVMe:
   ```yaml
   provision:
     nvme:
       swap_size_gb: 16
   ```

## SSH Connection Issues

### Connection refused

**Solutions:**
1. Verify SSH is running:
   ```bash
   sudo systemctl status ssh
   ```
2. Check firewall:
   ```bash
   sudo ufw status
   ```
3. Verify IP address is correct

### Authentication failed

**Solutions:**
1. Copy SSH key:
   ```bash
   ssh-copy-id nvidia@192.168.55.1
   ```
2. Check username (default is `nvidia`)

## Configuration Issues

### Invalid YAML configuration

**Symptoms:**
```
Error: InvalidYaml("expected ':', found ...")
```

**Solutions:**
1. Validate YAML syntax:
   ```bash
   python3 -c "import yaml; yaml.safe_load(open('cohete.yaml'))"
   ```
2. Check indentation (use spaces, not tabs)
3. Quote strings containing special characters

### Model source not found

**Symptoms:**
```
Error: Config("Model source not found: pacha://...")
```

**Solutions:**
1. Verify pacha server is running
2. Check model URL is correct
3. Ensure network connectivity to pacha server

## Performance Issues

### Slow inference

**Solutions:**
1. Enable jetson_clocks:
   ```rust
   let mut clocks = JetsonClocks::new();
   clocks.enable()?;
   ```
2. Use `PowerMode::Maxn` for maximum performance
3. Ensure NVMe swap is configured (not SD card)
4. Check for thermal throttling

### High latency between requests

**Solutions:**
1. Keep model loaded (don't reload per request)
2. Use batching
3. Check for memory pressure causing swapping

## Getting Help

1. Check [GitHub Issues](https://github.com/paiml/cohete/issues)
2. Run with debug logging:
   ```bash
   RUST_LOG=cohete=debug cargo run --example ...
   ```
3. Collect tegrastats output for bug reports:
   ```bash
   tegrastats --interval 1000 | tee tegrastats.log
   ```
