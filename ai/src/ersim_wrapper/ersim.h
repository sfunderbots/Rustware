#ifndef ERSIMCPP_H
#define ERSIMCPP_H


#include "rust/cxx.h"
#include <mtest.hpp>
#include <memory>

int ersquare(int x);

class SimulatorWrapper {
public:
    SimulatorWrapper();

    void set(int x) const;
    int get() const;

    Simulator simulator;
private:
    std::shared_ptr<Simulator> impl;
};

std::unique_ptr<SimulatorWrapper> new_simulator_wrapper();
#endif

