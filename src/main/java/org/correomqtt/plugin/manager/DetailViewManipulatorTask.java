package org.correomqtt.plugin.manager;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;
import org.correomqtt.plugin.spi.DetailViewManipulatorHook;

import java.util.List;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class DetailViewManipulatorTask {

    private String name;
    private List<DetailViewManipulatorHook> hooks;

}
