#include "rustware/src/ersim_wrapper/ersim.h"
#include "rustware/src/main.rs.h"
//#include <mtest.hpp>
#include <memory>
//#include <simulator.h>
//#include "simulator_wrapper.h"
//#include "simulator_wrapper.h"
//#include <simulator.h>
#include "simulator/simulator.h"
//#include <snappy.h>



int ersquare(int x) {
//    int foo = mysquarecpp(x);
//    auto foo = snappy::kBlockLog;
//    auto foooo = snappy::MaxCompressedLength(10);
//    auto foooo = snappy_max_compressed_length(10);
//    auto bar = kFoo;
//    auto baz = mysquarecpp2(4);
//    auto bbb = mysquarecpp(3);
    return x*x;
}

SimulatorWrapper::SimulatorWrapper(
        std::string geometry_config_absolute_filepath,
        std::string realism_config_absolute_filepath)
//        : impl(new underbots::ErsimWrapper(
//                geometry_config_absolute_filepath,
//                realism_config_absolute_filepath)
//                )
                {
                    auto foo = SerializedMsg();
                    auto bar = std::make_unique<camun::simulator::Simulator>("foo", "bar");
                }

//void SimulatorWrapper::set(int x) const{
////    impl->set_num(x);
//}
//
//int SimulatorWrapper::get() const {
//    return 1;
////    return impl->get_num();
//}

std::unique_ptr<SimulatorWrapper> new_simulator_wrapper() {
    return std::make_unique<SimulatorWrapper>("foo", "bar");
}
