# textfsm-rs
A one-long-weekend experiment in implementing TextFSM in Rust

The TextFSM itself is somewhat implemented, however the table is using unordered hashmaps, and a pretty hacky way to deal with the row keys.

However, it manages to extract the vast majority of information in ntc-templates tests:
```
NTC-TEMPLATES VERIFY RESULTS:
   Total tests run: 1549
      Could not load YAML: 0
      Verify success: 1537
      Results differ: 12
```

This is how to launch the test functionality (which aims to include the multi-template function),
and which tests are broken:

```
cargo run --release --example cli-table ~/network-automation/ntc-templates/

ntc-templates/tests/mikrotik_routeros/interface_print_detail/mikrotik_routeros_interface_print_detail_01.raw
ntc-templates/tests/cisco_ios/show_switch_detail/cisco_ios_show_switch_detail02.raw
ntc-templates/tests/cisco_ios/show_switch_detail/cisco_ios_show_switch_detail01.raw
ntc-templates/tests/cisco_ios/show_module/cisco_ios_show_module4.raw
ntc-templates/tests/cisco_ios/show_module/cisco_ios_show_module_02.raw
ntc-templates/tests/cisco_ios/show_module/cisco_ios_show_module1.raw
ntc-templates/tests/cisco_ios/show_module/cisco_ios_show_module5.raw
ntc-templates/tests/cisco_ios/show_module/cisco_ios_show_module_01.raw
ntc-templates/tests/cisco_ios/show_module/cisco_ios_show_module3.raw
ntc-templates/tests/cisco_ios/show_module/cisco_ios_show_module2.raw
ntc-templates/tests/huawei_smartax/display_ont_info_0/huawei_smartax_display_ont_info_fsp_4.raw
ntc-templates/tests/huawei_smartax/display_ont_info_summary_ont/huawei_smartax_display_ont_info_summary_ont_5.raw
```
