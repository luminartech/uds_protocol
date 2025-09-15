UDS Protocol Requirements
=========================

UDS, or Unified Diagnostic Services, is a protocol heavily used in the automotive industry.
UDS was originally developed for use on CAN (Controller Area Network) bus systems,
but it has since been adapted for use on other transport layers such as Ethernet, FlexRay, and LIN.
Defined in the ISO 14229 standard,
it provides a standardized way for diagnostic tools to communicate with vehicle electronic control units (ECUs).
In this context, "Diagnostics" refers to more than simply detecting and reporting ecu state and faults;
it encompasses a wide range of services including reading and clearing diagnostic trouble codes (DTCs),
accessing vehicle data, performing system tests, as well as configuring and reprogramming ECUs.

UDS Protocol is designed to provide an ergonomic set of tools for building software and tooling that works with UDS based ECUs.
The library provides strongly typed data structures for creating, sending, receiving, and parsing UDS messages independent of the transport layer.
UDS Protocol provides a simple set of traits allowing library users to implement custom diagnostic payloads.
Using these traits, users can create custom diagnostic identifiers, services,
and trouble codes that integrate seamlessly with the rest of the library.

.. toctree::
   :maxdepth: 2
   :caption: Contents:

   library_specification
