#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Ipv4Address, Stack, StackResources, ConfigV4, StaticConfigV4};
use embassy_stm32::eth::generic_smi::GenericSMI;
use embassy_stm32::eth::{Ethernet, PacketQueue};
use embassy_stm32::peripherals::ETH;
use embassy_stm32::rng::Rng;
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, eth, peripherals, rng, Config};
use embassy_time::Timer;
use embedded_io_async::Write;
use rand_core::RngCore;
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

type Device = Ethernet<'static, ETH, GenericSMI>;

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
async fn data_task(stack: &'static Stack<Device>) -> ! {
    let mut tcp_rx_buf = [0; 1024];
    let mut tcp_tx_buf = [0; 1024];

    let mut coms_socket = TcpSocket::new(stack, &mut tcp_rx_buf,  &mut tcp_tx_buf);
    coms_socket.set_timeout(Some(embassy_time::Duration::from_secs(1)));

    loop {
        info!("Listening on TCP:42171...");
        if let Err(_) = coms_socket.accept(42171).await {
            warn!("accept error");
            continue;
        }

        info!("Accepted a connection");

        let mut resp = [0; 16];
        resp[0] = 0xFF;

        // Write some quick output
        let r = coms_socket.write_all(&resp).await;
        if let Err(e) = r {
            warn!("write error: {:?}", e);
            // TODO handle this
        }

        let mut rx_buf = [0; 1024];
        let res = coms_socket.read(&mut rx_buf).await;
        if let Err(e) = res {
            warn!("error receiving data");
        } 

        Timer::after_millis(500).await;
        
        info!("Closing the connection");
        coms_socket.abort();
        info!("Flushing the RST out...");
        _ = coms_socket.flush().await;
        info!("Finished with the socket");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL216,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 216 / 2 = 216Mhz
            divq: None,
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    // Generate random seed.
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    let mac_addr = [0x00, 0x00, 0xDE, 0xAD, 0xBE, 0xEF];

    let device = Ethernet::new(
        make_static!(PacketQueue::<16, 16>::new()),
        p.ETH,
        Irqs,
        p.PA1,
        p.PA2,
        p.PC1,
        p.PA7,
        p.PC4,
        p.PC5,
        p.PG13,
        p.PB13,
        p.PG11,
        GenericSMI::new(0),
        mac_addr,
    );

    let config = embassy_net::Config::dhcpv4(Default::default());

    // Init network stack
    let stack = &*make_static!(Stack::new(
        device,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    // Launch network task
    unwrap!(spawner.spawn(net_task(stack)));
    unwrap!(spawner.spawn(data_task(stack)));

    // Ensure DHCP configuration is up before trying connect
    stack.wait_config_up().await;

    let assigned_ipv4 = stack.config_v4().unwrap();
    info!("network initialized, device up at {}", assigned_ipv4.address.address().as_bytes());


    // this might can be smaller
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 1024];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 1024];

    let mut discovery_socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    let discovery_endpoint = (Ipv4Address::new(239, 4, 21, 70), 42170);
    let _ = discovery_socket.bind(42170);

    // send heartbeats telling the upstream computer where it can find us permanently
    loop {
        info!("connecting...");
        let mut buf = [0; 16];
        buf[0] = assigned_ipv4.address.address().as_bytes()[0];
        buf[1] = assigned_ipv4.address.address().as_bytes()[1];
        buf[2] = assigned_ipv4.address.address().as_bytes()[2];
        buf[3] = assigned_ipv4.address.address().as_bytes()[3];
        buf[4] = 0xA4; // top byte of port 42171
        buf[5] = 0xBB; // top byte of port 42171
        buf[6] = 0; // unit/backplane (determine via hardware)
        buf[7] = 0; // slot number (determine via hardware)
        // TODO add some status/health checks

        loop {
            discovery_socket.send_to(&buf[..16], discovery_endpoint).await.unwrap();
            Timer::after_millis(1000).await;
        }
    }
}