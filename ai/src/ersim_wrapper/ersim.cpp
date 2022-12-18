#include "rustware/src/ersim_wrapper/ersim.h"
#include "rustware/src/main.rs.h"
#include <mtest.hpp>
#include <memory>
//#include <snappy.h>



int ersquare(int x) {
//    int foo = mysquarecpp(x);
//    auto foo = snappy::kBlockLog;
//    auto foooo = snappy::MaxCompressedLength(10);
//    auto foooo = snappy_max_compressed_length(10);
//    auto bar = kFoo;
//    auto baz = mysquarecpp2(4);
    auto bbb = mysquarecpp(3);
    return x*x;
}

SimulatorWrapper::SimulatorWrapper() : impl(new Simulator()) {}

void SimulatorWrapper::set(int x) const{
    impl->set_num(x);
}

int SimulatorWrapper::get() const {
    return impl->get_num();
}

std::unique_ptr<SimulatorWrapper> new_simulator_wrapper() {
    return std::make_unique<SimulatorWrapper>();
}
