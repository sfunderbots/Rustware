#ifndef ERSIMCPP_H
#define ERSIMCPP_H


#include "rust/cxx.h"
//#include <mtest.hpp>
#include <memory>

int ersquare(int x);

//namespace underbots {
//    class ErsimWrapper;
//}

class SimulatorWrapper {
public:
    SimulatorWrapper(std::string geometry_config_absolute_filepath, std::string realism_config_absolute_filepath);

//    void set(int x) const;
//    int get() const;

//    Simulator simulator;
private:
//    std::shared_ptr<underbots::ErsimWrapper> impl;
};

std::unique_ptr<SimulatorWrapper> new_simulator_wrapper();
#endif

